use crate::board::Position;
use crate::movegen::MoveList;
use crate::types::{Move, Piece};

const PIECE_VALUES: [i32; 6] = [100, 320, 330, 500, 900, 20000];

pub struct KillerTable {
    pub moves: [[Move; 2]; 256], // MAX_PLY = 256
}

impl Default for KillerTable {
    fn default() -> Self {
        Self::new()
    }
}

impl KillerTable {
    pub fn new() -> Self {
        Self {
            moves: [[Move::NONE; 2]; 256],
        }
    }

    pub fn store(&mut self, ply: usize, mv: Move) {
        if self.moves[ply][0] != mv {
            self.moves[ply][1] = self.moves[ply][0];
            self.moves[ply][0] = mv;
        }
    }
}

pub struct HistoryTable {
    pub table: [[[i32; 64]; 64]; 2],
}

impl Default for HistoryTable {
    fn default() -> Self {
        Self::new()
    }
}

impl HistoryTable {
    pub fn new() -> Self {
        Self {
            table: [[[0; 64]; 64]; 2],
        }
    }

    pub fn record(&mut self, side: usize, from: usize, to: usize, depth: i32) {
        let bonus = (depth * depth).min(400);
        let entry = &mut self.table[side][from][to];
        *entry = (*entry + bonus).min(600_000);
    }

    pub fn score(&self, side: usize, from: usize, to: usize) -> i32 {
        self.table[side][from][to]
    }
}

pub fn order_moves(
    moves: &mut MoveList,
    pv_move: Move,
    killers: &[Move; 2],
    history: &HistoryTable,
    pos: &Position,
) {
    let mut scores = [0i32; 256];

    for i in 0..moves.len() {
        let mv = moves[i];
        let score = if mv == pv_move {
            10_000_000
        } else if let Some((_, victim)) = pos.piece_at(mv.to_sq()) {
            let (_, attacker) = pos.piece_at(mv.from_sq()).unwrap();
            1_000_000 + PIECE_VALUES[victim as usize] * 10 - PIECE_VALUES[attacker as usize]
        } else if mv.is_en_passant() {
            1_000_000 + PIECE_VALUES[Piece::Pawn as usize] * 10 - PIECE_VALUES[Piece::Pawn as usize]
        } else if mv.is_promotion() {
            900_000 + PIECE_VALUES[mv.promotion_piece() as usize]
        } else if mv == killers[0] {
            800_000
        } else if mv == killers[1] {
            700_000
        } else {
            history
                .score(
                    pos.side_to_move() as usize,
                    mv.from_sq().0 as usize,
                    mv.to_sq().0 as usize,
                )
                .clamp(1, 600_000)
        };

        scores[i] = score;
    }

    // Sort moves based on scores in descending order.
    // We use a simple stable sort or just zip and sort.
    let mut combined: Vec<(Move, i32)> = moves
        .as_slice()
        .iter()
        .zip(scores.iter())
        .map(|(&m, &s)| (m, s))
        .collect();

    combined.sort_by_key(|&(_, s)| -s);

    for (i, (m, _)) in combined.into_iter().enumerate() {
        moves.as_mut_slice()[i] = m;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Position;
    use crate::movegen::generate_moves;
    use crate::types::Square;

    #[test]
    fn test_order_pv_move_first() {
        crate::movegen::magics::init();
        let pos = Position::startpos();
        let mut moves = generate_moves(&pos);
        let pv_move = Move::new(Square(12), Square(28), 0, 0); // e2e4

        order_moves(
            &mut moves,
            pv_move,
            &[Move::NONE, Move::NONE],
            &HistoryTable::default(),
            &pos,
        );
        assert_eq!(moves[0], pv_move);
    }

    #[test]
    fn test_mvv_lva() {
        crate::movegen::magics::init();
        // Position where white pawn at d3 can capture queen at c4 and knight at e4
        let pos = Position::from_fen("k7/8/8/8/2q1n3/3P4/8/K7 w - - 0 1").unwrap();
        let mut moves = generate_moves(&pos);

        order_moves(
            &mut moves,
            Move::NONE,
            &[Move::NONE, Move::NONE],
            &HistoryTable::default(),
            &pos,
        );

        // PxQ (d3xc4) should come before PxN (d3xe4)
        let pxc4 = Move::new(
            Square::from_file_rank(3, 2),
            Square::from_file_rank(2, 3),
            0,
            0,
        );
        let pxe4 = Move::new(
            Square::from_file_rank(3, 2),
            Square::from_file_rank(4, 3),
            0,
            0,
        );

        let mut found_pxc4 = false;
        let mut found_pxe4 = false;

        for i in 0..moves.len() {
            if moves[i] == pxc4 {
                assert!(!found_pxe4, "PxQ should come before PxN");
                found_pxc4 = true;
            } else if moves[i] == pxe4 {
                found_pxe4 = true;
            }
        }
        assert!(found_pxc4);
        assert!(found_pxe4);
    }

    #[test]
    fn test_history_ordering() {
        crate::movegen::magics::init();
        let pos = Position::startpos();
        let mut moves = generate_moves(&pos);

        // Pick two quiet moves: g1f3 and b1c3
        let g1f3 = Move::new(Square(6), Square(21), 0, 0);
        let b1c3 = Move::new(Square(1), Square(18), 0, 0);

        let mut history = HistoryTable::new();
        // Record g1f3 to give it a high score
        history.record(0, 6, 21, 10);

        order_moves(
            &mut moves,
            Move::NONE,
            &[Move::NONE, Move::NONE],
            &history,
            &pos,
        );

        assert_eq!(moves[0], g1f3);
        // b1c3 should be somewhere in the list
        let mut found_b1c3 = false;
        for i in 0..moves.len() {
            if moves[i] == b1c3 {
                found_b1c3 = true;
                break;
            }
        }
        assert!(found_b1c3);
    }
}
