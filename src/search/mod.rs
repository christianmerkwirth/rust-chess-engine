pub mod alphabeta;
pub mod ordering;
pub mod smp;
pub mod tt;

use crate::board::Position;
use crate::search::alphabeta::{search, SearchState, INFINITY};
use crate::search::tt::TranspositionTable;
use crate::tablebase::SyzygyTablebase;
use crate::types::Move;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

#[derive(Clone, Debug, Default)]
pub struct SearchLimits {
    pub depth: Option<i32>,
    pub movetime: Option<u64>,
    pub infinite: bool,
    pub ponder: bool,
}

pub struct SearchInfo {
    pub depth: i32,
    pub nodes: u64,
    pub score: i32,
    pub time: u128,
    pub pv: Vec<Move>,
}

pub struct SearchResult {
    pub best_move: Move,
    pub score: i32,
    pub ponder_move: Option<Move>,
}

#[allow(clippy::too_many_arguments)]
pub fn iterative_deepening(
    pos: &Position,
    tt: &TranspositionTable,
    limits: &SearchLimits,
    stop: &AtomicBool,
    pondering: &AtomicBool,
    tablebase: Option<Arc<SyzygyTablebase>>,
    start_depth: i32,
    info_callback: impl Fn(SearchInfo),
) -> SearchResult {
    let start_time = Instant::now();
    let mut state = SearchState::new(stop, pondering, limits.movetime);
    state.tablebase = tablebase;
    let mut best_score = 0;
    let mut best_move = {
        let moves = crate::movegen::generate_moves(pos);
        if moves.is_empty() {
            Move::NONE
        } else {
            moves[0]
        }
    };
    let mut ponder_move = None;

    let max_depth = limits.depth.unwrap_or(100);

    for depth in start_depth..=max_depth {
        let score = search(pos, &mut state, tt, depth, 0, -INFINITY, INFINITY);

        if stop.load(Ordering::Relaxed) {
            // Only update best move if we actually found something
            if let Some(data) = tt.probe(pos.hash()) {
                if data.best_move != Move::NONE {
                    best_move = data.best_move;
                    best_score = data.score as i32;
                }
            }
            break;
        }

        best_score = score;
        if let Some(data) = tt.probe(pos.hash()) {
            best_move = data.best_move;
        }

        let pv = get_pv(pos, tt, depth);
        if pv.len() >= 2 {
            ponder_move = Some(pv[1]);
        } else {
            ponder_move = None;
        }

        let info = SearchInfo {
            depth,
            nodes: state.nodes,
            score: best_score,
            time: start_time.elapsed().as_millis(),
            pv,
        };
        info_callback(info);

        if stop.load(Ordering::Relaxed) {
            break;
        }

        // If we found mate, we can stop
        if best_score.abs() > alphabeta::MATE_SCORE - 1000 {
            break;
        }
    }

    SearchResult {
        best_move,
        score: best_score,
        ponder_move,
    }
}

fn get_pv(pos: &Position, tt: &TranspositionTable, depth: i32) -> Vec<Move> {
    let mut pv = Vec::new();
    let mut current_pos = pos.clone();

    for _ in 0..depth {
        if let Some(data) = tt.probe(current_pos.hash()) {
            let mv = data.best_move;
            if mv == Move::NONE {
                break;
            }

            // Simple legality check
            let moves = crate::movegen::generate_moves(&current_pos);
            let mut found = false;
            for i in 0..moves.len() {
                if moves[i] == mv {
                    found = true;
                    break;
                }
            }
            if !found {
                break;
            }

            pv.push(mv);
            current_pos.make_move(mv);
        } else {
            break;
        }
    }
    pv
}
