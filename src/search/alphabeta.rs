use crate::board::Position;
use crate::eval::evaluate;
use crate::movegen::{generate_captures, generate_moves};
use crate::search::ordering::{order_moves, KillerTable};
use crate::search::tt::{TTData, TTFlag, TranspositionTable};
use crate::tablebase::{SyzygyTablebase, WdlResult};
use crate::types::{Move, Piece};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

pub const MATE_SCORE: i32 = 30000;
pub const INFINITY: i32 = 32000;
/// Score used for tablebase wins/losses. Below mate, above normal eval range.
pub const TABLEBASE_WIN: i32 = 20000;

#[inline(always)]
pub fn score_to_tt(s: i32, ply: i32) -> i16 {
    if s >= MATE_SCORE - 1000 {
        (s + ply) as i16
    } else if s <= -MATE_SCORE + 1000 {
        (s - ply) as i16
    } else {
        s as i16
    }
}

#[inline(always)]
pub fn score_from_tt(s: i16, ply: i32) -> i32 {
    let s = s as i32;
    if s >= MATE_SCORE - 1000 {
        s - ply
    } else if s <= -MATE_SCORE + 1000 {
        s + ply
    } else {
        s
    }
}

use std::time::Instant;

pub struct SearchState<'a> {
    pub killers: KillerTable,
    pub nodes: &'a AtomicU64,
    pub stop: &'a AtomicBool,
    pub pondering: &'a AtomicBool,
    pub start_time: Instant,
    pub movetime: Option<u64>,
    pub nodes_limit: Option<u64>,
    pub tablebase: Option<Arc<SyzygyTablebase>>,
    pub was_pondering: bool,
}

impl<'a> SearchState<'a> {
    pub fn new(
        stop: &'a AtomicBool,
        pondering: &'a AtomicBool,
        nodes: &'a AtomicU64,
        movetime: Option<u64>,
        nodes_limit: Option<u64>,
    ) -> Self {
        Self {
            killers: KillerTable::new(),
            nodes,
            stop,
            pondering,
            start_time: Instant::now(),
            movetime,
            nodes_limit,
            tablebase: None,
            was_pondering: pondering.load(Ordering::Relaxed),
        }
    }
}

pub fn search(
    pos: &Position,
    state: &mut SearchState,
    tt: &TranspositionTable,
    depth: i32,
    ply: usize,
    mut alpha: i32,
    mut beta: i32,
) -> i32 {
    let nodes = state.nodes.fetch_add(1, Ordering::Relaxed) + 1;

    // Check stop flag every 2048 nodes
    if nodes & 0x7FF == 0 {
        if state.stop.load(Ordering::Relaxed) {
            return 0;
        }

        let currently_pondering = state.pondering.load(Ordering::Relaxed);
        if state.was_pondering && !currently_pondering {
            state.start_time = Instant::now();
            state.was_pondering = false;
        }

        if !currently_pondering {
            if let Some(limit) = state.movetime {
                let elapsed = state.start_time.elapsed().as_millis();
                if elapsed >= limit as u128 {
                    state.stop.store(true, Ordering::Relaxed);
                    return 0;
                }
            }

            if let Some(limit) = state.nodes_limit {
                if nodes >= limit {
                    state.stop.store(true, Ordering::Relaxed);
                    return 0;
                }
            }
        }
    }

    // Check for draw
    if pos.is_draw_by_fifty()
        || (ply > 0
            && (pos.is_draw_by_repetition() || pos.is_draw_by_insufficient_material()))
    {
        return 0;
    }

    // TT Probe
    let tt_hit = tt.probe(pos.hash());
    let mut pv_move = Move::NONE;
    if let Some(data) = tt_hit {
        pv_move = data.best_move;
        if data.depth >= depth as i8 && ply > 0 {
            let score = score_from_tt(data.score, ply as i32);
            match data.flag {
                TTFlag::Exact => return score,
                TTFlag::LowerBound => {
                    if score >= beta {
                        return score;
                    }
                    alpha = alpha.max(score);
                }
                TTFlag::UpperBound => {
                    if score <= alpha {
                        return score;
                    }
                    beta = beta.min(score);
                }
            }
        }
    }

    // Tablebase probe
    if let Some(ref tb) = state.tablebase {
        if pos.occupancy().count() <= 6 {
            if let Some(wdl) = tb.probe_wdl(pos) {
                let score = match wdl {
                    WdlResult::Win => TABLEBASE_WIN - ply as i32,
                    WdlResult::Loss => -TABLEBASE_WIN + ply as i32,
                    WdlResult::CursedWin => 1,
                    WdlResult::BlessedLoss => -1,
                    WdlResult::Draw => 0,
                };
                return score;
            }
        }
    }

    // Null Move Pruning (NMP)
    if depth >= 3
        && !pos.is_in_check(pos.side_to_move())
        && ply > 0 // Don't do NMP at the root
    {
        let us = pos.side_to_move();
        // Simple condition: only if we have more than just pawns
        let non_pawn_pieces = pos.occupancy_color(us)
            & !(pos.pieces(us, Piece::Pawn) | pos.pieces(us, Piece::King));
        if !non_pawn_pieces.is_empty() {
            let mut next_pos = pos.clone();
            next_pos.make_null_move();
            // Reduction: R=2 or 3
            let r = 3;
            let score = -search(&next_pos, state, tt, depth - 1 - r, ply + 1, -beta, -beta + 1);

            if score >= beta {
                return beta;
            }
        }
    }

    if depth <= 0 {
        return quiescence(pos, state, alpha, beta, 0);
    }

    let mut moves = generate_moves(pos);
    if moves.is_empty() {
        if pos.is_in_check(pos.side_to_move()) {
            return -MATE_SCORE + ply as i32;
        } else {
            return 0; // Stalemate
        }
    }

    order_moves(&mut moves, pv_move, &state.killers.moves[ply], pos);

    let mut best_move = Move::NONE;
    let mut best_score = -INFINITY;
    let old_alpha = alpha;

    for i in 0..moves.len() {
        let mv = moves[i];
        let mut next_pos = pos.clone();
        next_pos.make_move(mv);

        let mut score;
        if i == 0 {
            // Full window search for first move (PV move)
            score = -search(&next_pos, state, tt, depth - 1, ply + 1, -beta, -alpha);
        } else {
            // Null window search for non-PV moves
            score = -search(&next_pos, state, tt, depth - 1, ply + 1, -alpha - 1, -alpha);
            if score > alpha && score < beta {
                // Re-search with full window if it might be a new best move
                score = -search(&next_pos, state, tt, depth - 1, ply + 1, -beta, -alpha);
            }
        }

        if state.stop.load(Ordering::Relaxed) {
            return 0;
        }

        if score > best_score {
            best_score = score;
            best_move = mv;

            if score > alpha {
                alpha = score;
                if score >= beta {
                    // Killer move
                    if !mv.is_capture_or_promotion(pos) {
                        state.killers.store(ply, mv);
                    }
                    break;
                }
            }
        }
    }

    // TT Store
    let flag = if best_score >= beta {
        TTFlag::LowerBound
    } else if best_score > old_alpha {
        TTFlag::Exact
    } else {
        TTFlag::UpperBound
    };

    tt.store(
        pos.hash(),
        TTData {
            depth: depth as i8,
            score: score_to_tt(best_score, ply as i32),
            flag,
            best_move,
            gen: 0, // Will be set by tt.store
        },
    );

    best_score
}

pub const MAX_QPLY: usize = 64;

pub fn quiescence(
    pos: &Position,
    state: &mut SearchState,
    mut alpha: i32,
    beta: i32,
    qply: usize,
) -> i32 {
    let nodes = state.nodes.fetch_add(1, Ordering::Relaxed) + 1;

    // Check stop flag every 2048 nodes
    if nodes & 0x7FF == 0 {
        if state.stop.load(Ordering::Relaxed) {
            return 0;
        }

        if !state.pondering.load(Ordering::Relaxed) {
            if let Some(limit) = state.movetime {
                let elapsed = state.start_time.elapsed().as_millis();
                if elapsed >= limit as u128 {
                    state.stop.store(true, Ordering::Relaxed);
                    return 0;
                }
            }

            if let Some(limit) = state.nodes_limit {
                if nodes >= limit {
                    state.stop.store(true, Ordering::Relaxed);
                    return 0;
                }
            }
        }
    }

    // FR-17: Ply bound for quiescence
    if qply >= MAX_QPLY {
        return evaluate(pos);
    }

    let stand_pat = evaluate(pos);
    if stand_pat >= beta {
        return stand_pat;
    }
    if stand_pat > alpha {
        alpha = stand_pat;
    }

    let mut moves = generate_captures(pos);
    // Move ordering for captures (MVV-LVA)
    order_moves(&mut moves, Move::NONE, &[Move::NONE, Move::NONE], pos);

    for i in 0..moves.len() {
        let mv = moves[i];
        let mut next_pos = pos.clone();
        next_pos.make_move(mv);

        let score = -quiescence(&next_pos, state, -beta, -alpha, qply + 1);

        if state.stop.load(Ordering::Relaxed) {
            return 0;
        }

        if score >= beta {
            return score;
        }
        if score > alpha {
            alpha = score;
        }
    }

    alpha
}

trait MoveExt {
    fn is_capture_or_promotion(&self, pos: &Position) -> bool;
}

impl MoveExt for Move {
    fn is_capture_or_promotion(&self, pos: &Position) -> bool {
        self.is_promotion() || pos.piece_at(self.to_sq()).is_some() || self.is_en_passant()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Square;

    #[test]
    fn test_search_finds_mate_in_1() {
        crate::movegen::magics::init();
        // White to move, mate in 1 with Ra8#
        let pos = Position::from_fen("4k3/R7/4K3/8/8/8/8/8 w - - 0 1").unwrap();
        let tt = TranspositionTable::new(1);
        let stop = AtomicBool::new(false);
        let pondering = AtomicBool::new(false);
        let nodes = AtomicU64::new(0);
        let mut state = SearchState::new(&stop, &pondering, &nodes, None, None);

        let score = search(&pos, &mut state, &tt, 2, 0, -INFINITY, INFINITY);
        assert!(score > MATE_SCORE - 10);

        let data = tt.probe(pos.hash()).unwrap();
        assert_eq!(data.best_move.from_sq(), Square::from_file_rank(0, 6)); // a7
        assert_eq!(data.best_move.to_sq(), Square::from_file_rank(0, 7)); // a8
    }

    #[test]
    fn test_search_finds_forced_mate_in_1() {
        crate::movegen::magics::init();
        // White to move, mate in 1 with Qf7#
        let pos = Position::from_fen(
            "r1bqkb1r/pppp1ppp/2n2n2/4p2Q/2B1P3/8/PPPP1PPP/RNB1K1NR w KQkq - 0 1",
        )
        .unwrap();
        let tt = TranspositionTable::new(1);
        let stop = AtomicBool::new(false);
        let pondering = AtomicBool::new(false);
        let nodes = AtomicU64::new(0);
        let mut state = SearchState::new(&stop, &pondering, &nodes, None, None);

        let score = search(&pos, &mut state, &tt, 2, 0, -INFINITY, INFINITY);
        assert!(score > MATE_SCORE - 10);

        let data = tt.probe(pos.hash()).unwrap();
        assert_eq!(data.best_move.from_sq(), Square::from_file_rank(7, 4)); // h5
        assert_eq!(data.best_move.to_sq(), Square::from_file_rank(5, 6)); // f7
    }

    #[test]
    fn test_mate_score_conversion() {
        let ply = 5;
        let score = MATE_SCORE - 10;
        let tt_score = score_to_tt(score, ply);
        assert_eq!(tt_score, (score + ply) as i16);
        assert_eq!(score_from_tt(tt_score, ply), score);

        let score = -MATE_SCORE + 10;
        let tt_score = score_to_tt(score, ply);
        assert_eq!(tt_score, (score - ply) as i16);
        assert_eq!(score_from_tt(tt_score, ply), score);

        let score = 500;
        let tt_score = score_to_tt(score, ply);
        assert_eq!(tt_score, score as i16);
        assert_eq!(score_from_tt(tt_score, ply), score);
    }
}
