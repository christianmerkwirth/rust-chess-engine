pub mod pst;

use crate::board::Position;
use crate::types::{Color, Piece};
use pst::{MATERIAL_EG, MATERIAL_MG, PST_EG, PST_MG};

/// Evaluate position in centipawns from side-to-move perspective.
pub fn evaluate(pos: &Position) -> i32 {
    let mut mg = [0, 0]; // [White, Black]
    let mut eg = [0, 0];

    for color in [Color::White, Color::Black] {
        let c_idx = color as usize;
        for piece in [
            Piece::Pawn,
            Piece::Knight,
            Piece::Bishop,
            Piece::Rook,
            Piece::Queen,
            Piece::King,
        ] {
            let p_idx = piece as usize;
            let mut bb = pos.pieces(color, piece);
            while !bb.is_empty() {
                let sq = bb.pop_lsb();

                // Mirror square for black pieces (PSTs are from white's perspective)
                let eval_sq = if color == Color::White {
                    sq.0 as usize
                } else {
                    (sq.0 ^ 56) as usize
                };

                mg[c_idx] += MATERIAL_MG[p_idx] + PST_MG[p_idx][eval_sq];
                eg[c_idx] += MATERIAL_EG[p_idx] + PST_EG[p_idx][eval_sq];
            }
        }
    }

    let mg_score = mg[0] - mg[1];
    let eg_score = eg[0] - eg[1];

    let phase = compute_phase(pos);

    // Tapered evaluation: interpolate between middlegame and endgame
    let score = (mg_score * phase + eg_score * (24 - phase)) / 24;

    if pos.side_to_move() == Color::White {
        score
    } else {
        -score
    }
}

/// Game phase score: 0 (endgame) to 24 (opening).
/// Phase values: P=0, N=1, B=1, R=2, Q=4.
pub fn compute_phase(pos: &Position) -> i32 {
    let mut phase = 0;
    phase += pos.pieces(Color::White, Piece::Knight).count() as i32;
    phase += pos.pieces(Color::Black, Piece::Knight).count() as i32;
    phase += pos.pieces(Color::White, Piece::Bishop).count() as i32;
    phase += pos.pieces(Color::Black, Piece::Bishop).count() as i32;
    phase += pos.pieces(Color::White, Piece::Rook).count() as i32 * 2;
    phase += pos.pieces(Color::Black, Piece::Rook).count() as i32 * 2;
    phase += pos.pieces(Color::White, Piece::Queen).count() as i32 * 4;
    phase += pos.pieces(Color::Black, Piece::Queen).count() as i32 * 4;

    // Ensure phase is within [0, 24]
    phase.min(24)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_phase_startpos() {
        let pos = Position::startpos();
        assert_eq!(compute_phase(&pos), 24);
    }

    #[test]
    fn test_compute_phase_endgame() {
        // King + Pawn vs King
        let pos = Position::from_fen("8/8/8/8/8/4k3/4P3/4K3 w - - 0 1").unwrap();
        assert_eq!(compute_phase(&pos), 0);
    }

    #[test]
    fn test_evaluate_startpos() {
        let pos = Position::startpos();
        // Starting position should be roughly balanced, but usually slightly
        // favors white due to PSTs (e.g. center control).
        let eval = evaluate(&pos);
        assert!(eval.abs() < 50);
    }

    #[test]
    fn test_evaluate_material_advantage() {
        // White has an extra queen
        let pos =
            Position::from_fen("rnb1kbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        let eval = evaluate(&pos);
        assert!(eval > 800);
    }

    #[test]
    fn test_evaluate_symmetry() {
        let pos_w =
            Position::from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1")
                .unwrap();
        let pos_b =
            Position::from_fen("rnbqkbnr/pppp1ppp/8/4p3/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1")
                .unwrap();

        // These positions are mirrors of each other, so evaluation should be the same
        // from the side-to-move perspective.
        assert_eq!(evaluate(&pos_w), evaluate(&pos_b));
    }

    #[test]
    fn test_evaluate_side_to_move() {
        let pos = Position::startpos();
        let eval_w = evaluate(&pos);

        // Flip side to move without changing pieces
        let pos_b =
            Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1").unwrap();

        assert_eq!(eval_w, -evaluate(&pos_b));
    }
}
