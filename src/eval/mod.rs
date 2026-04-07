pub mod pst;

use crate::board::Position;
use crate::types::{Color, Piece};
use pst::{MATERIAL_EG, MATERIAL_MG, PST_EG, PST_MG};

pub const BISHOP_PAIR_MG: i32 = 30;
pub const BISHOP_PAIR_EG: i32 = 50;

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

                // Mirror square for white pieces to match BERF (visual) PST tables.
                // For black, the square is already in the correct relative rank.
                let eval_sq = if color == Color::White {
                    (sq.0 ^ 56) as usize
                } else {
                    sq.0 as usize
                };

                mg[c_idx] += MATERIAL_MG[p_idx] + PST_MG[p_idx][eval_sq];
                eg[c_idx] += MATERIAL_EG[p_idx] + PST_EG[p_idx][eval_sq];
            }
        }

        // Bishop pair bonus
        if pos.pieces(color, Piece::Bishop).count() >= 2 {
            mg[c_idx] += BISHOP_PAIR_MG;
            eg[c_idx] += BISHOP_PAIR_EG;
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
        crate::movegen::magics::init();
        let pos = Position::startpos();
        // Starting position should be roughly balanced, but usually slightly
        // favors white due to PSTs (e.g. center control).
        let eval = evaluate(&pos);
        assert!(eval.abs() < 50);
    }

    #[test]
    fn test_evaluate_material_advantage() {
        crate::movegen::magics::init();
        // White has an extra queen
        let pos =
            Position::from_fen("rnb1kbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        let eval = evaluate(&pos);
        assert!(eval > 800);
    }

    #[test]
    fn test_evaluate_symmetry() {
        crate::movegen::magics::init();
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
        crate::movegen::magics::init();
        let pos = Position::startpos();
        let eval_w = evaluate(&pos);

        // Flip side to move without changing pieces
        let pos_b =
            Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1").unwrap();

        assert_eq!(eval_w, -evaluate(&pos_b));
    }

    #[test]
    fn test_eval_e2e4_improves_white() {
        crate::movegen::magics::init();
        let pos_start = Position::startpos();
        let pos_e2e4 = Position::from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1").unwrap();
        
        let eval_start = evaluate(&pos_start); // White to move
        // evaluate returns side-to-move perspective. 
        // For pos_e2e4, it's black to move, so we negate it to get white's score.
        let eval_e2e4 = -evaluate(&pos_e2e4); 
        
        assert!(eval_e2e4 > eval_start, "e2e4 eval ({}) should be better than startpos ({}) from white's POV", eval_e2e4, eval_start);
    }

    #[test]
    fn test_eval_a7_beats_a2() {
        crate::movegen::magics::init();
        // One white pawn on a7 vs one white pawn on a2
        let pos_a2 = Position::from_fen("8/8/8/8/8/8/P7/4K2k w - - 0 1").unwrap();
        let pos_a7 = Position::from_fen("8/P7/8/8/8/8/8/4K2k w - - 0 1").unwrap();
        
        let eval_a2 = evaluate(&pos_a2);
        let eval_a7 = evaluate(&pos_a7);
        
        assert!(eval_a7 > eval_a2, "Pawn on a7 ({}) should be better than on a2 ({})", eval_a7, eval_a2);
    }

    #[test]
    fn test_king_mg_prefers_back_rank() {
        crate::movegen::magics::init();
        // Middlegame phase (many pieces)
        let fen_e1 = "rnbq1rk1/pppp1ppp/4pn2/8/8/4PN2/PPPP1PPP/RNBQK2R w KQ - 0 1";
        let fen_e4 = "rnbq1rk1/pppp1ppp/4pn2/8/4K3/4PN2/PPPP1PPP/RNBQ3R w - - 0 1";
        let fen_g1 = "rnbq1rk1/pppp1ppp/4pn2/8/8/4PN2/PPPP1PPP/RNBQ1RK1 w - - 0 1";

        let pos_e1 = Position::from_fen(fen_e1).unwrap();
        let pos_e4 = Position::from_fen(fen_e4).unwrap();
        let pos_g1 = Position::from_fen(fen_g1).unwrap();

        let eval_e1 = evaluate(&pos_e1);
        let eval_e4 = evaluate(&pos_e4);
        let eval_g1 = evaluate(&pos_g1);

        assert!(eval_e1 > eval_e4, "King on e1 ({}) should be better than on e4 ({})", eval_e1, eval_e4);
        assert!(eval_g1 > eval_e4, "King on g1 ({}) should be better than on e4 ({})", eval_g1, eval_e4);
    }

    #[test]
    fn test_pst_bishop_row7_not_duplicate() {
        // Bishop MG row 0 (BERF rank 8) and row 7 (BERF rank 1) should not be identical
        let row0 = &PST_MG[Piece::Bishop as usize][0..8];
        let row7 = &PST_MG[Piece::Bishop as usize][56..64];
        assert_ne!(row0, row7, "Bishop MG row 0 and row 7 should not be identical");
    }

    #[test]
    fn test_material_mg_ne_eg() {
        assert_ne!(MATERIAL_MG, MATERIAL_EG, "MG and EG material values should be different");
    }

    #[test]
    fn test_bishop_pair_bonus() {
        crate::movegen::magics::init();
        // White has two bishops, black has one.
        // Positions are otherwise symmetrical.
        let pos = Position::from_fen("k7/8/8/8/8/8/B1B5/K6b w - - 0 1").unwrap();
        let eval = evaluate(&pos);
        
        // White has two bishops: MG bonus 30, EG bonus 50.
        // Black has one bishop: no bonus.
        // Plus PST and material differences.
        
        // Let's compare with a position where white only has one bishop.
        let pos_one = Position::from_fen("k7/8/8/8/8/8/B7/K6b w - - 0 1").unwrap();
        let eval_one = evaluate(&pos_one);
        
        assert!(eval > eval_one + 30, "Bishop pair should provide a significant bonus");
    }
}
