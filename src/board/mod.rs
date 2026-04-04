pub mod zobrist;

use crate::bitboard::Bitboard;
use crate::types::{CastlingRights, Color, Move, Piece, Square};

#[derive(Debug, PartialEq, Eq)]
pub enum FenError {
    InvalidFormat,
    InvalidPiecePlacement,
    InvalidSideToMove,
    InvalidCastlingRights,
    InvalidEnPassant,
    InvalidClocks,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Position {
    piece_bb: [Bitboard; 12],
    color_bb: [Bitboard; 2],
    side: Color,
    castling: CastlingRights,
    en_passant: Option<Square>,
    halfmove_clock: u32,
    fullmove_number: u32,
    hash: u64,
}

impl Position {
    pub fn startpos() -> Position {
        todo!()
    }

    pub fn from_fen(_fen: &str) -> Result<Position, FenError> {
        todo!()
    }

    pub fn to_fen(&self) -> String {
        todo!()
    }

    pub fn make_move(&mut self, _mv: Move) {
        todo!()
    }

    pub fn is_in_check(&self, _color: Color) -> bool {
        todo!()
    }

    pub fn piece_at(&self, _sq: Square) -> Option<(Color, Piece)> {
        todo!()
    }

    pub fn pieces(&self, color: Color, piece: Piece) -> Bitboard {
        self.piece_bb[color as usize * 6 + piece as usize]
    }

    pub fn occupancy(&self) -> Bitboard {
        self.color_bb[0] | self.color_bb[1]
    }

    pub fn side_to_move(&self) -> Color {
        self.side
    }

    pub fn castling_rights(&self) -> CastlingRights {
        self.castling
    }

    pub fn en_passant_square(&self) -> Option<Square> {
        self.en_passant
    }

    pub fn hash(&self) -> u64 {
        self.hash
    }

    pub fn is_draw_by_fifty(&self) -> bool {
        self.halfmove_clock >= 100
    }

    pub fn compute_hash(&self) -> u64 {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitboard::Bitboard;

    fn p(fen: &str) -> Position {
        Position::from_fen(fen).unwrap()
    }

    // --- startpos ---

    #[test]
    fn test_startpos_side_to_move() {
        assert_eq!(Position::startpos().side_to_move(), Color::White);
    }

    #[test]
    fn test_startpos_castling_rights() {
        let r = Position::startpos().castling_rights();
        assert!(r.has(CastlingRights::WK));
        assert!(r.has(CastlingRights::WQ));
        assert!(r.has(CastlingRights::BK));
        assert!(r.has(CastlingRights::BQ));
    }

    #[test]
    fn test_startpos_white_pawns() {
        assert_eq!(
            Position::startpos().pieces(Color::White, Piece::Pawn),
            Bitboard(0x0000_0000_0000_FF00)
        );
    }

    #[test]
    fn test_startpos_black_pawns() {
        assert_eq!(
            Position::startpos().pieces(Color::Black, Piece::Pawn),
            Bitboard(0x00FF_0000_0000_0000)
        );
    }

    #[test]
    fn test_startpos_white_rooks() {
        assert_eq!(
            Position::startpos().pieces(Color::White, Piece::Rook),
            Bitboard(0x0000_0000_0000_0081)
        );
    }

    #[test]
    fn test_startpos_white_king() {
        assert_eq!(
            Position::startpos().pieces(Color::White, Piece::King),
            Bitboard(0x0000_0000_0000_0010)
        );
    }

    #[test]
    fn test_startpos_black_king() {
        assert_eq!(
            Position::startpos().pieces(Color::Black, Piece::King),
            Bitboard(0x1000_0000_0000_0000)
        );
    }

    #[test]
    fn test_startpos_occupancy() {
        assert_eq!(
            Position::startpos().occupancy(),
            Bitboard(0xFFFF_0000_0000_FFFF)
        );
    }

    #[test]
    fn test_startpos_no_check() {
        let pos = Position::startpos();
        assert!(!pos.is_in_check(Color::White));
        assert!(!pos.is_in_check(Color::Black));
    }

    // --- FEN parsing ---

    #[test]
    fn test_from_fen_matches_startpos() {
        assert_eq!(
            Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap(),
            Position::startpos()
        );
    }

    #[test]
    fn test_from_fen_after_e4() {
        let pos = p("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1");
        assert_eq!(pos.side_to_move(), Color::Black);
        assert_eq!(pos.en_passant_square(), Some(Square::from_file_rank(4, 2)));
        assert!(pos.pieces(Color::White, Piece::Pawn).is_set(Square::from_file_rank(4, 3)));
    }

    #[test]
    fn test_from_fen_no_castling() {
        let pos = p("8/8/8/8/8/8/8/4K3 w - - 0 1");
        assert_eq!(pos.castling_rights(), CastlingRights(0));
    }

    #[test]
    fn test_from_fen_partial_castling() {
        let pos = p("r3k2r/8/8/8/8/8/8/R3K2R w Kq - 0 1");
        let r = pos.castling_rights();
        assert!(r.has(CastlingRights::WK));
        assert!(!r.has(CastlingRights::WQ));
        assert!(!r.has(CastlingRights::BK));
        assert!(r.has(CastlingRights::BQ));
    }

    #[test]
    fn test_from_fen_halfmove_not_draw() {
        let pos = p("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 5 10");
        assert!(!pos.is_draw_by_fifty());
    }

    #[test]
    fn test_from_fen_invalid_short() {
        assert!(Position::from_fen("invalid").is_err());
    }

    #[test]
    fn test_from_fen_invalid_side() {
        assert!(Position::from_fen(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR x KQkq - 0 1"
        )
        .is_err());
    }

    // --- to_fen round-trip ---

    #[test]
    fn test_to_fen_startpos() {
        assert_eq!(
            Position::startpos().to_fen(),
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        );
    }

    #[test]
    fn test_to_fen_roundtrip() {
        let fens = [
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1",
            "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1",
            "8/8/8/8/8/8/8/4K3 w - - 0 1",
        ];
        for fen in &fens {
            let pos = Position::from_fen(fen).unwrap();
            assert_eq!(&pos.to_fen(), fen, "Round-trip failed for: {}", fen);
        }
    }

    // --- piece_at ---

    #[test]
    fn test_piece_at_startpos() {
        let pos = Position::startpos();
        assert_eq!(pos.piece_at(Square(0)), Some((Color::White, Piece::Rook)));
        assert_eq!(pos.piece_at(Square(4)), Some((Color::White, Piece::King)));
        assert_eq!(pos.piece_at(Square(60)), Some((Color::Black, Piece::King)));
        assert_eq!(pos.piece_at(Square(27)), None);
    }

    // --- make_move: quiet / pawn ---

    #[test]
    fn test_make_move_quiet_pawn() {
        let mut pos = Position::startpos();
        let mv = Move::new(Square::from_file_rank(4, 1), Square::from_file_rank(4, 2), 0, 0);
        pos.make_move(mv);
        assert!(pos.pieces(Color::White, Piece::Pawn).is_set(Square::from_file_rank(4, 2)));
        assert!(!pos.pieces(Color::White, Piece::Pawn).is_set(Square::from_file_rank(4, 1)));
        assert_eq!(pos.side_to_move(), Color::Black);
    }

    #[test]
    fn test_make_move_double_pawn_push_sets_ep() {
        let mut pos = Position::startpos();
        let mv = Move::new(Square::from_file_rank(4, 1), Square::from_file_rank(4, 3), 0, 0);
        pos.make_move(mv);
        assert_eq!(pos.en_passant_square(), Some(Square::from_file_rank(4, 2)));
    }

    #[test]
    fn test_make_move_single_push_clears_ep() {
        let mut pos = p("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1");
        let mv = Move::new(Square::from_file_rank(3, 6), Square::from_file_rank(3, 5), 0, 0);
        pos.make_move(mv);
        assert_eq!(pos.en_passant_square(), None);
    }

    // --- make_move: capture ---

    #[test]
    fn test_make_move_capture() {
        let mut pos = p("rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2");
        let mv = Move::new(Square::from_file_rank(4, 3), Square::from_file_rank(3, 4), 0, 0);
        pos.make_move(mv);
        assert!(pos.pieces(Color::White, Piece::Pawn).is_set(Square::from_file_rank(3, 4)));
        assert!(!pos.pieces(Color::Black, Piece::Pawn).is_set(Square::from_file_rank(3, 4)));
    }

    // --- make_move: en passant ---

    #[test]
    fn test_make_move_en_passant() {
        // e5 pawn captures d6 en passant (black just pushed d7-d5)
        let mut pos =
            p("rnbqkbnr/ppp1pppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 3");
        let mv = Move::new(Square::from_file_rank(4, 4), Square::from_file_rank(3, 5), 2, 0);
        pos.make_move(mv);
        assert!(pos.pieces(Color::White, Piece::Pawn).is_set(Square::from_file_rank(3, 5)));
        assert!(!pos.pieces(Color::Black, Piece::Pawn).is_set(Square::from_file_rank(3, 4)));
        assert!(!pos.pieces(Color::White, Piece::Pawn).is_set(Square::from_file_rank(4, 4)));
    }

    // --- make_move: castling ---

    #[test]
    fn test_make_move_castling_wk() {
        let mut pos = p("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1");
        pos.make_move(Move::new(Square(4), Square(6), 3, 0));
        assert!(pos.pieces(Color::White, Piece::King).is_set(Square(6)));
        assert!(pos.pieces(Color::White, Piece::Rook).is_set(Square(5)));
        assert!(!pos.pieces(Color::White, Piece::King).is_set(Square(4)));
        assert!(!pos.pieces(Color::White, Piece::Rook).is_set(Square(7)));
        let r = pos.castling_rights();
        assert!(!r.has(CastlingRights::WK));
        assert!(!r.has(CastlingRights::WQ));
        assert!(r.has(CastlingRights::BK));
        assert!(r.has(CastlingRights::BQ));
    }

    #[test]
    fn test_make_move_castling_wq() {
        let mut pos = p("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1");
        pos.make_move(Move::new(Square(4), Square(2), 3, 0));
        assert!(pos.pieces(Color::White, Piece::King).is_set(Square(2)));
        assert!(pos.pieces(Color::White, Piece::Rook).is_set(Square(3)));
        assert!(!pos.pieces(Color::White, Piece::King).is_set(Square(4)));
        assert!(!pos.pieces(Color::White, Piece::Rook).is_set(Square(0)));
    }

    #[test]
    fn test_make_move_castling_bk() {
        let mut pos = p("r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1");
        pos.make_move(Move::new(Square(60), Square(62), 3, 0));
        assert!(pos.pieces(Color::Black, Piece::King).is_set(Square(62)));
        assert!(pos.pieces(Color::Black, Piece::Rook).is_set(Square(61)));
    }

    #[test]
    fn test_make_move_castling_bq() {
        let mut pos = p("r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1");
        pos.make_move(Move::new(Square(60), Square(58), 3, 0));
        assert!(pos.pieces(Color::Black, Piece::King).is_set(Square(58)));
        assert!(pos.pieces(Color::Black, Piece::Rook).is_set(Square(59)));
    }

    // --- make_move: promotion ---

    #[test]
    fn test_make_move_promotion_queen() {
        let mut pos = p("8/P7/8/8/8/8/8/4K3 w - - 0 1");
        pos.make_move(Move::new(
            Square::from_file_rank(0, 6),
            Square::from_file_rank(0, 7),
            1,
            3,
        ));
        assert!(pos.pieces(Color::White, Piece::Queen).is_set(Square::from_file_rank(0, 7)));
        assert!(!pos.pieces(Color::White, Piece::Pawn).is_set(Square::from_file_rank(0, 6)));
    }

    #[test]
    fn test_make_move_promotion_knight() {
        let mut pos = p("8/P7/8/8/8/8/8/4K3 w - - 0 1");
        pos.make_move(Move::new(
            Square::from_file_rank(0, 6),
            Square::from_file_rank(0, 7),
            1,
            0,
        ));
        assert!(pos.pieces(Color::White, Piece::Knight).is_set(Square::from_file_rank(0, 7)));
    }

    #[test]
    fn test_make_move_promotion_rook() {
        let mut pos = p("8/P7/8/8/8/8/8/4K3 w - - 0 1");
        pos.make_move(Move::new(
            Square::from_file_rank(0, 6),
            Square::from_file_rank(0, 7),
            1,
            2,
        ));
        assert!(pos.pieces(Color::White, Piece::Rook).is_set(Square::from_file_rank(0, 7)));
    }

    #[test]
    fn test_make_move_promotion_bishop() {
        let mut pos = p("8/P7/8/8/8/8/8/4K3 w - - 0 1");
        pos.make_move(Move::new(
            Square::from_file_rank(0, 6),
            Square::from_file_rank(0, 7),
            1,
            1,
        ));
        assert!(pos.pieces(Color::White, Piece::Bishop).is_set(Square::from_file_rank(0, 7)));
    }

    // --- halfmove clock ---

    #[test]
    fn test_halfmove_clock_pawn_resets() {
        let mut pos = p("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 5 3");
        pos.make_move(Move::new(
            Square::from_file_rank(4, 1),
            Square::from_file_rank(4, 2),
            0,
            0,
        ));
        let fen = pos.to_fen();
        let parts: Vec<&str> = fen.split(' ').collect();
        assert_eq!(parts[4], "0");
    }

    #[test]
    fn test_halfmove_clock_capture_resets() {
        let mut pos =
            p("rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 5 2");
        pos.make_move(Move::new(
            Square::from_file_rank(4, 3),
            Square::from_file_rank(3, 4),
            0,
            0,
        ));
        let fen = pos.to_fen();
        let parts: Vec<&str> = fen.split(' ').collect();
        assert_eq!(parts[4], "0");
    }

    #[test]
    fn test_halfmove_clock_increments_on_quiet() {
        let mut pos = p("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 3 1");
        // Ra1-a2: quiet rook move
        pos.make_move(Move::new(Square(0), Square(8), 0, 0));
        let fen = pos.to_fen();
        let parts: Vec<&str> = fen.split(' ').collect();
        assert_eq!(parts[4], "4");
    }

    // --- castling rights revoked by king/rook move ---

    #[test]
    fn test_castling_rights_king_move() {
        let mut pos = p("8/8/8/8/8/8/8/4K3 w KQ - 0 1");
        pos.make_move(Move::new(Square(4), Square(12), 0, 0)); // e1-e2
        let r = pos.castling_rights();
        assert!(!r.has(CastlingRights::WK));
        assert!(!r.has(CastlingRights::WQ));
    }

    #[test]
    fn test_castling_rights_rook_move_wk() {
        let mut pos = p("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1");
        pos.make_move(Move::new(Square(7), Square(15), 0, 0)); // Rh1-h2
        let r = pos.castling_rights();
        assert!(!r.has(CastlingRights::WK));
        assert!(r.has(CastlingRights::WQ));
    }

    #[test]
    fn test_castling_rights_rook_captured() {
        // White captures black's a8 rook → BQ right revoked
        let mut pos = p("r3k3/8/8/8/8/8/8/R3K3 w Q - 0 1");
        pos.make_move(Move::new(Square(0), Square(56), 0, 0)); // Ra1xa8
        let r = pos.castling_rights();
        assert!(!r.has(CastlingRights::BQ));
    }

    // --- is_draw_by_fifty ---

    #[test]
    fn test_is_draw_by_fifty_at_100() {
        assert!(p("8/8/8/8/8/8/8/4K3 w - - 100 60").is_draw_by_fifty());
    }

    #[test]
    fn test_is_not_draw_at_99() {
        assert!(!p("8/8/8/8/8/8/8/4K3 w - - 99 60").is_draw_by_fifty());
    }

    // --- is_in_check ---

    #[test]
    fn test_in_check_pawn() {
        // Black pawn on d5 attacks white king on e4
        let pos = p("8/8/8/3p4/4K3/8/8/4k3 w - - 0 1");
        assert!(pos.is_in_check(Color::White));
    }

    #[test]
    fn test_in_check_knight() {
        // Black knight on f6 attacks white king on e4
        let pos = p("8/8/5n2/8/4K3/8/8/4k3 w - - 0 1");
        assert!(pos.is_in_check(Color::White));
    }

    #[test]
    fn test_in_check_bishop() {
        // Black bishop on b7 diagonally attacks white king on e4
        let pos = p("8/1b6/8/8/4K3/8/8/4k3 w - - 0 1");
        assert!(pos.is_in_check(Color::White));
    }

    #[test]
    fn test_in_check_rook() {
        // Black rook on e8 attacks white king on e4
        let pos = p("4r3/8/8/8/4K3/8/8/4k3 w - - 0 1");
        assert!(pos.is_in_check(Color::White));
    }

    #[test]
    fn test_in_check_queen_rank() {
        // Black queen on h4 attacks white king on e4 along rank
        let pos = p("8/8/8/8/4K2q/8/8/4k3 w - - 0 1");
        assert!(pos.is_in_check(Color::White));
    }

    #[test]
    fn test_in_check_blocked() {
        // White pawn on e6 blocks black rook on e8 from attacking white king on e4
        let pos = p("4r3/8/4P3/8/4K3/8/8/4k3 w - - 0 1");
        assert!(!pos.is_in_check(Color::White));
    }

    #[test]
    fn test_not_in_check_startpos() {
        let pos = Position::startpos();
        assert!(!pos.is_in_check(Color::White));
        assert!(!pos.is_in_check(Color::Black));
    }

    // --- Zobrist hash ---

    #[test]
    fn test_zobrist_incremental_matches_full_after_move() {
        let mut pos = Position::startpos();
        pos.make_move(Move::new(
            Square::from_file_rank(4, 1),
            Square::from_file_rank(4, 3),
            0,
            0,
        ));
        assert_eq!(pos.hash(), pos.compute_hash());
    }

    #[test]
    fn test_zobrist_different_positions_differ() {
        let pos1 =
            p("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1");
        let pos2 =
            p("rnbqkbnr/pppppppp/8/8/3P4/8/PPP1PPPP/RNBQKBNR b KQkq d3 0 1");
        assert_ne!(pos1.hash(), pos2.hash());
    }

    #[test]
    fn test_zobrist_same_position_same_hash() {
        let pos1 =
            p("rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2");
        let pos2 =
            p("rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2");
        assert_eq!(pos1.hash(), pos2.hash());
    }

    #[test]
    fn test_zobrist_incremental_after_capture() {
        let mut pos = p("rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2");
        pos.make_move(Move::new(
            Square::from_file_rank(4, 3),
            Square::from_file_rank(3, 4),
            0,
            0,
        ));
        assert_eq!(pos.hash(), pos.compute_hash());
    }

    #[test]
    fn test_zobrist_incremental_after_castling() {
        let mut pos = p("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1");
        pos.make_move(Move::new(Square(4), Square(6), 3, 0));
        assert_eq!(pos.hash(), pos.compute_hash());
    }

    #[test]
    fn test_zobrist_incremental_after_en_passant() {
        let mut pos = p("rnbqkbnr/ppp1pppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 3");
        pos.make_move(Move::new(
            Square::from_file_rank(4, 4),
            Square::from_file_rank(3, 5),
            2,
            0,
        ));
        assert_eq!(pos.hash(), pos.compute_hash());
    }

    #[test]
    fn test_zobrist_incremental_after_promotion() {
        let mut pos = p("8/P7/8/8/8/8/8/4K3 w - - 0 1");
        pos.make_move(Move::new(
            Square::from_file_rank(0, 6),
            Square::from_file_rank(0, 7),
            1,
            3,
        ));
        assert_eq!(pos.hash(), pos.compute_hash());
    }
}
