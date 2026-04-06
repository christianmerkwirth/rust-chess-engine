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

/// Full board state: 12 piece bitboards, 2 color occupancy bitboards, side to
/// move, castling rights, en-passant square, clocks, and Zobrist hash.
///
/// Indexed as `piece_bb[color as usize * 6 + piece as usize]`.
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

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

fn piece_from_char(c: char) -> Option<(Color, Piece)> {
    match c {
        'P' => Some((Color::White, Piece::Pawn)),
        'N' => Some((Color::White, Piece::Knight)),
        'B' => Some((Color::White, Piece::Bishop)),
        'R' => Some((Color::White, Piece::Rook)),
        'Q' => Some((Color::White, Piece::Queen)),
        'K' => Some((Color::White, Piece::King)),
        'p' => Some((Color::Black, Piece::Pawn)),
        'n' => Some((Color::Black, Piece::Knight)),
        'b' => Some((Color::Black, Piece::Bishop)),
        'r' => Some((Color::Black, Piece::Rook)),
        'q' => Some((Color::Black, Piece::Queen)),
        'k' => Some((Color::Black, Piece::King)),
        _ => None,
    }
}

fn piece_to_char(color: Color, piece: Piece) -> char {
    let c = match piece {
        Piece::Pawn => 'p',
        Piece::Knight => 'n',
        Piece::Bishop => 'b',
        Piece::Rook => 'r',
        Piece::Queen => 'q',
        Piece::King => 'k',
    };
    if color == Color::White {
        c.to_ascii_uppercase()
    } else {
        c
    }
}

/// Squares a knight on `sq` can reach.
fn knight_attacks(sq: Square) -> Bitboard {
    let mut result = Bitboard::empty();
    let f = sq.file() as i32;
    let r = sq.rank() as i32;
    for &(df, dr) in &[
        (2, 1),
        (2, -1),
        (-2, 1),
        (-2, -1),
        (1, 2),
        (1, -2),
        (-1, 2),
        (-1, -2_i32),
    ] {
        let nf = f + df;
        let nr = r + dr;
        if (0..8).contains(&nf) && (0..8).contains(&nr) {
            result.set(Square::from_file_rank(nf as u8, nr as u8));
        }
    }
    result
}

/// Squares a king on `sq` can reach (adjacent squares).
fn king_attacks(sq: Square) -> Bitboard {
    let mut result = Bitboard::empty();
    let f = sq.file() as i32;
    let r = sq.rank() as i32;
    for &(df, dr) in &[
        (1, 0),
        (-1, 0),
        (0, 1),
        (0, -1),
        (1, 1),
        (1, -1),
        (-1, 1),
        (-1, -1_i32),
    ] {
        let nf = f + df;
        let nr = r + dr;
        if (0..8).contains(&nf) && (0..8).contains(&nr) {
            result.set(Square::from_file_rank(nf as u8, nr as u8));
        }
    }
    result
}

/// Squares attacked BY a pawn of `color` standing on `sq`.
fn pawn_attacks(sq: Square, color: Color) -> Bitboard {
    let mut result = Bitboard::empty();
    let f = sq.file() as i32;
    let r = sq.rank() as i32;
    let dr: i32 = if color == Color::White { 1 } else { -1 };
    for df in &[-1i32, 1] {
        let nf = f + df;
        let nr = r + dr;
        if (0..8).contains(&nf) && (0..8).contains(&nr) {
            result.set(Square::from_file_rank(nf as u8, nr as u8));
        }
    }
    result
}

/// All squares reachable from `sq` along `deltas`, stopping when blocked by
/// an occupied square (the occupied square is included — it may be capturable).
fn sliding_attack(sq: Square, occ: Bitboard, deltas: &[(i32, i32)]) -> Bitboard {
    let mut result = Bitboard::empty();
    let file0 = sq.file() as i32;
    let rank0 = sq.rank() as i32;
    for &(df, dr) in deltas {
        let mut f = file0 + df;
        let mut r = rank0 + dr;
        while (0..8).contains(&f) && (0..8).contains(&r) {
            let target = Square::from_file_rank(f as u8, r as u8);
            result.set(target);
            if occ.is_set(target) {
                break;
            }
            f += df;
            r += dr;
        }
    }
    result
}

// ---------------------------------------------------------------------------
// Position implementation
// ---------------------------------------------------------------------------

impl Position {
    /// Standard chess starting position.
    pub fn startpos() -> Position {
        Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }

    /// Parse a FEN string into a `Position`.
    pub fn from_fen(fen: &str) -> Result<Position, FenError> {
        let parts: Vec<&str> = fen.split_whitespace().collect();
        if parts.len() < 4 {
            return Err(FenError::InvalidFormat);
        }

        // --- piece placement ---
        let mut piece_bb = [Bitboard::empty(); 12];
        let mut color_bb = [Bitboard::empty(); 2];

        let ranks: Vec<&str> = parts[0].split('/').collect();
        if ranks.len() != 8 {
            return Err(FenError::InvalidPiecePlacement);
        }
        for (rank_idx, rank_str) in ranks.iter().enumerate() {
            let rank = 7 - rank_idx as u8;
            let mut file: u8 = 0;
            for c in rank_str.chars() {
                if c.is_ascii_digit() {
                    let n = c as u8 - b'0';
                    file += n;
                    if file > 8 {
                        return Err(FenError::InvalidPiecePlacement);
                    }
                } else if let Some((color, piece)) = piece_from_char(c) {
                    if file >= 8 {
                        return Err(FenError::InvalidPiecePlacement);
                    }
                    let sq = Square::from_file_rank(file, rank);
                    piece_bb[color as usize * 6 + piece as usize].set(sq);
                    color_bb[color as usize].set(sq);
                    file += 1;
                } else {
                    return Err(FenError::InvalidPiecePlacement);
                }
            }
            if file != 8 {
                return Err(FenError::InvalidPiecePlacement);
            }
        }

        // --- side to move ---
        let side = match parts[1] {
            "w" => Color::White,
            "b" => Color::Black,
            _ => return Err(FenError::InvalidSideToMove),
        };

        // --- castling rights ---
        let castling = if parts[2] == "-" {
            CastlingRights(0)
        } else {
            let mut rights = 0u8;
            for c in parts[2].chars() {
                match c {
                    'K' => rights |= CastlingRights::WK,
                    'Q' => rights |= CastlingRights::WQ,
                    'k' => rights |= CastlingRights::BK,
                    'q' => rights |= CastlingRights::BQ,
                    _ => return Err(FenError::InvalidCastlingRights),
                }
            }
            CastlingRights(rights)
        };

        // --- en passant ---
        let en_passant = if parts[3] == "-" {
            None
        } else if parts[3].len() == 2 {
            let bytes = parts[3].as_bytes();
            let file = bytes[0].wrapping_sub(b'a');
            let rank = bytes[1].wrapping_sub(b'1');
            if file >= 8 || rank >= 8 {
                return Err(FenError::InvalidEnPassant);
            }
            Some(Square::from_file_rank(file, rank))
        } else {
            return Err(FenError::InvalidEnPassant);
        };

        // --- clocks (optional; default 0/1) ---
        let halfmove_clock = if parts.len() > 4 {
            parts[4]
                .parse::<u32>()
                .map_err(|_| FenError::InvalidClocks)?
        } else {
            0
        };
        let fullmove_number = if parts.len() > 5 {
            parts[5]
                .parse::<u32>()
                .map_err(|_| FenError::InvalidClocks)?
        } else {
            1
        };

        let mut pos = Position {
            piece_bb,
            color_bb,
            side,
            castling,
            en_passant,
            halfmove_clock,
            fullmove_number,
            hash: 0,
        };
        pos.hash = pos.compute_hash();
        Ok(pos)
    }

    /// Serialize the position to a FEN string.
    pub fn to_fen(&self) -> String {
        let mut fen = String::new();

        // piece placement (rank 8 down to rank 1)
        for rank in (0u8..8).rev() {
            let mut empty = 0u8;
            for file in 0u8..8 {
                let sq = Square::from_file_rank(file, rank);
                if let Some((color, piece)) = self.piece_at(sq) {
                    if empty > 0 {
                        fen.push(char::from(b'0' + empty));
                        empty = 0;
                    }
                    fen.push(piece_to_char(color, piece));
                } else {
                    empty += 1;
                }
            }
            if empty > 0 {
                fen.push(char::from(b'0' + empty));
            }
            if rank > 0 {
                fen.push('/');
            }
        }

        // side to move
        fen.push(' ');
        fen.push(if self.side == Color::White { 'w' } else { 'b' });

        // castling
        fen.push(' ');
        if self.castling.0 == 0 {
            fen.push('-');
        } else {
            if self.castling.has(CastlingRights::WK) {
                fen.push('K');
            }
            if self.castling.has(CastlingRights::WQ) {
                fen.push('Q');
            }
            if self.castling.has(CastlingRights::BK) {
                fen.push('k');
            }
            if self.castling.has(CastlingRights::BQ) {
                fen.push('q');
            }
        }

        // en passant
        fen.push(' ');
        if let Some(ep) = self.en_passant {
            fen.push(char::from(b'a' + ep.file()));
            fen.push(char::from(b'1' + ep.rank()));
        } else {
            fen.push('-');
        }

        // clocks
        fen.push(' ');
        fen.push_str(&self.halfmove_clock.to_string());
        fen.push(' ');
        fen.push_str(&self.fullmove_number.to_string());

        fen
    }

    /// Apply `mv` to the position (copy-make model: clone before calling if
    /// you need to restore the original).
    ///
    /// Precondition: `mv` is pseudo-legal for this position.
    /// Postcondition: side to move flipped, Zobrist hash updated incrementally.
    pub fn make_move(&mut self, mv: Move) {
        let from = mv.from_sq();
        let to = mv.to_sq();
        let us = self.side;
        let them = us.opposite();
        let (_, moving_piece) = self.piece_at(from).expect("no piece at from-square");

        // XOR out old castling and en-passant contributions
        self.hash ^= zobrist::castling_key(self.castling);
        if let Some(ep) = self.en_passant {
            self.hash ^= zobrist::en_passant_key(ep.file());
        }

        match mv.flags() {
            // -----------------------------------------------------------------
            // Normal move (quiet or capture)
            // -----------------------------------------------------------------
            0 => {
                if let Some((_, cap)) = self.piece_at(to) {
                    let ci = them as usize * 6 + cap as usize;
                    self.piece_bb[ci].clear(to);
                    self.color_bb[them as usize].clear(to);
                    self.hash ^= zobrist::piece_key(them, cap, to);
                    self.halfmove_clock = 0;
                } else if moving_piece == Piece::Pawn {
                    self.halfmove_clock = 0;
                } else {
                    self.halfmove_clock += 1;
                }

                let mi = us as usize * 6 + moving_piece as usize;
                self.piece_bb[mi].clear(from);
                self.piece_bb[mi].set(to);
                self.color_bb[us as usize].clear(from);
                self.color_bb[us as usize].set(to);
                self.hash ^= zobrist::piece_key(us, moving_piece, from);
                self.hash ^= zobrist::piece_key(us, moving_piece, to);

                // Set or clear en-passant square
                let rank_diff = to.rank() as i32 - from.rank() as i32;
                self.en_passant = if moving_piece == Piece::Pawn && rank_diff == 2 {
                    Some(Square::from_file_rank(from.file(), from.rank() + 1))
                } else if moving_piece == Piece::Pawn && rank_diff == -2 {
                    Some(Square::from_file_rank(from.file(), from.rank() - 1))
                } else {
                    None
                };
            }

            // -----------------------------------------------------------------
            // Promotion (with optional capture)
            // -----------------------------------------------------------------
            1 => {
                let promo = mv.promotion_piece();

                if let Some((_, cap)) = self.piece_at(to) {
                    let ci = them as usize * 6 + cap as usize;
                    self.piece_bb[ci].clear(to);
                    self.color_bb[them as usize].clear(to);
                    self.hash ^= zobrist::piece_key(them, cap, to);
                }
                self.halfmove_clock = 0;

                let pawn_i = us as usize * 6 + Piece::Pawn as usize;
                let promo_i = us as usize * 6 + promo as usize;
                self.piece_bb[pawn_i].clear(from);
                self.piece_bb[promo_i].set(to);
                self.color_bb[us as usize].clear(from);
                self.color_bb[us as usize].set(to);
                self.hash ^= zobrist::piece_key(us, Piece::Pawn, from);
                self.hash ^= zobrist::piece_key(us, promo, to);

                self.en_passant = None;
            }

            // -----------------------------------------------------------------
            // En-passant capture
            // -----------------------------------------------------------------
            2 => {
                // Captured pawn is on the same rank as the moving pawn, same
                // file as the destination.
                let cap_sq = Square::from_file_rank(to.file(), from.rank());

                let cap_i = them as usize * 6 + Piece::Pawn as usize;
                self.piece_bb[cap_i].clear(cap_sq);
                self.color_bb[them as usize].clear(cap_sq);
                self.hash ^= zobrist::piece_key(them, Piece::Pawn, cap_sq);

                let pawn_i = us as usize * 6 + Piece::Pawn as usize;
                self.piece_bb[pawn_i].clear(from);
                self.piece_bb[pawn_i].set(to);
                self.color_bb[us as usize].clear(from);
                self.color_bb[us as usize].set(to);
                self.hash ^= zobrist::piece_key(us, Piece::Pawn, from);
                self.hash ^= zobrist::piece_key(us, Piece::Pawn, to);

                self.halfmove_clock = 0;
                self.en_passant = None;
            }

            // -----------------------------------------------------------------
            // Castling (king moves; also move the rook)
            // -----------------------------------------------------------------
            3 => {
                let (rook_from, rook_to): (Square, Square) = match to.0 {
                    6 => (Square(7), Square(5)),    // WK: h1→f1
                    2 => (Square(0), Square(3)),    // WQ: a1→d1
                    62 => (Square(63), Square(61)), // BK: h8→f8
                    58 => (Square(56), Square(59)), // BQ: a8→d8
                    _ => unreachable!("invalid castling target"),
                };

                // Move king
                let ki = us as usize * 6 + Piece::King as usize;
                self.piece_bb[ki].clear(from);
                self.piece_bb[ki].set(to);
                self.color_bb[us as usize].clear(from);
                self.color_bb[us as usize].set(to);
                self.hash ^= zobrist::piece_key(us, Piece::King, from);
                self.hash ^= zobrist::piece_key(us, Piece::King, to);

                // Move rook
                let ri = us as usize * 6 + Piece::Rook as usize;
                self.piece_bb[ri].clear(rook_from);
                self.piece_bb[ri].set(rook_to);
                self.color_bb[us as usize].clear(rook_from);
                self.color_bb[us as usize].set(rook_to);
                self.hash ^= zobrist::piece_key(us, Piece::Rook, rook_from);
                self.hash ^= zobrist::piece_key(us, Piece::Rook, rook_to);

                self.halfmove_clock += 1;
                self.en_passant = None;
            }

            _ => unreachable!(),
        }

        // --- update castling rights ---
        // King move: revoke both rights for that color
        if moving_piece == Piece::King {
            match us {
                Color::White => self
                    .castling
                    .remove(CastlingRights::WK | CastlingRights::WQ),
                Color::Black => self
                    .castling
                    .remove(CastlingRights::BK | CastlingRights::BQ),
            }
        }
        // Rook origin squares: touching them (move or capture) revokes the right
        let rook_squares: [(u8, u8); 4] = [
            (0, CastlingRights::WQ),
            (7, CastlingRights::WK),
            (56, CastlingRights::BQ),
            (63, CastlingRights::BK),
        ];
        for &(sq_idx, flag) in &rook_squares {
            if from.0 == sq_idx || to.0 == sq_idx {
                self.castling.remove(flag);
            }
        }

        // XOR in new castling and en-passant contributions
        self.hash ^= zobrist::castling_key(self.castling);
        if let Some(ep) = self.en_passant {
            self.hash ^= zobrist::en_passant_key(ep.file());
        }

        // Flip side to move
        self.hash ^= zobrist::side_key();
        if us == Color::Black {
            self.fullmove_number += 1;
        }
        self.side = them;
    }

    /// Returns true if `color`'s king is currently in check.
    /// Uses simple ray tracing — does not require magic bitboards.
    pub fn is_in_check(&self, color: Color) -> bool {
        let king_bb = self.pieces(color, Piece::King);
        if king_bb.is_empty() {
            return false;
        }
        let king_sq = king_bb.lsb();
        let occ = self.occupancy();
        let enemy = color.opposite();

        // Pawn attacks (use king's color so we look diagonally toward enemy)
        if !(pawn_attacks(king_sq, color) & self.pieces(enemy, Piece::Pawn)).is_empty() {
            return true;
        }

        // Knight attacks
        if !(knight_attacks(king_sq) & self.pieces(enemy, Piece::Knight)).is_empty() {
            return true;
        }

        // Enemy king (adjacency — needed for legality, not check-giving in practice)
        if !(king_attacks(king_sq) & self.pieces(enemy, Piece::King)).is_empty() {
            return true;
        }

        // Diagonal sliders (bishop / queen)
        let enemy_diag = self.pieces(enemy, Piece::Bishop) | self.pieces(enemy, Piece::Queen);
        if !(sliding_attack(king_sq, occ, &[(1, 1), (1, -1), (-1, 1), (-1, -1)]) & enemy_diag)
            .is_empty()
        {
            return true;
        }

        // Straight sliders (rook / queen)
        let enemy_straight = self.pieces(enemy, Piece::Rook) | self.pieces(enemy, Piece::Queen);
        if !(sliding_attack(king_sq, occ, &[(1, 0), (-1, 0), (0, 1), (0, -1)]) & enemy_straight)
            .is_empty()
        {
            return true;
        }

        false
    }

    /// What piece (if any) occupies `sq`?
    pub fn piece_at(&self, sq: Square) -> Option<(Color, Piece)> {
        use Piece::*;
        for color in [Color::White, Color::Black] {
            for piece in [Pawn, Knight, Bishop, Rook, Queen, King] {
                if self.piece_bb[color as usize * 6 + piece as usize].is_set(sq) {
                    return Some((color, piece));
                }
            }
        }
        None
    }

    /// Bitboard for a given color/piece combination.
    pub fn pieces(&self, color: Color, piece: Piece) -> Bitboard {
        self.piece_bb[color as usize * 6 + piece as usize]
    }

    /// All occupied squares.
    pub fn occupancy(&self) -> Bitboard {
        self.color_bb[0] | self.color_bb[1]
    }

    pub fn occupancy_color(&self, color: Color) -> Bitboard {
        self.color_bb[color as usize]
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

    /// Make a "null move" (passing the turn to the opponent).
    /// Used for Null Move Pruning in search.
    pub fn make_null_move(&mut self) {
        // XOR out old EP
        if let Some(ep) = self.en_passant {
            self.hash ^= zobrist::en_passant_key(ep.file());
        }
        self.en_passant = None;

        // Flip side to move
        self.hash ^= zobrist::side_key();
        if self.side == Color::Black {
            self.fullmove_number += 1;
        }
        self.side = self.side.opposite();

        self.halfmove_clock += 1;
    }

    pub fn is_draw_by_fifty(&self) -> bool {
        self.halfmove_clock >= 100
    }

    /// Compute Zobrist hash from scratch (used to verify incremental updates).
    pub fn compute_hash(&self) -> u64 {
        use Piece::*;
        let mut h = 0u64;
        for color in [Color::White, Color::Black] {
            for piece in [Pawn, Knight, Bishop, Rook, Queen, King] {
                let mut bb = self.pieces(color, piece);
                while !bb.is_empty() {
                    h ^= zobrist::piece_key(color, piece, bb.pop_lsb());
                }
            }
        }
        h ^= zobrist::castling_key(self.castling);
        if let Some(ep) = self.en_passant {
            h ^= zobrist::en_passant_key(ep.file());
        }
        if self.side == Color::Black {
            h ^= zobrist::side_key();
        }
        h
    }

    /// Convert a UCI move string (e.g., "e2e4", "e7e8q") to a `Move` in this
    /// position.
    pub fn parse_move(&self, uci: &str) -> Option<Move> {
        let moves = crate::movegen::generate_moves(self);
        (&moves).into_iter().find(|&m| m.to_uci() == uci)
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
        assert!(pos
            .pieces(Color::White, Piece::Pawn)
            .is_set(Square::from_file_rank(4, 3)));
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
        assert!(
            Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR x KQkq - 0 1").is_err()
        );
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
        let mv = Move::new(
            Square::from_file_rank(4, 1),
            Square::from_file_rank(4, 2),
            0,
            0,
        );
        pos.make_move(mv);
        assert!(pos
            .pieces(Color::White, Piece::Pawn)
            .is_set(Square::from_file_rank(4, 2)));
        assert!(!pos
            .pieces(Color::White, Piece::Pawn)
            .is_set(Square::from_file_rank(4, 1)));
        assert_eq!(pos.side_to_move(), Color::Black);
    }

    #[test]
    fn test_make_move_double_pawn_push_sets_ep() {
        let mut pos = Position::startpos();
        let mv = Move::new(
            Square::from_file_rank(4, 1),
            Square::from_file_rank(4, 3),
            0,
            0,
        );
        pos.make_move(mv);
        assert_eq!(pos.en_passant_square(), Some(Square::from_file_rank(4, 2)));
    }

    #[test]
    fn test_make_move_single_push_clears_ep() {
        let mut pos = p("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1");
        let mv = Move::new(
            Square::from_file_rank(3, 6),
            Square::from_file_rank(3, 5),
            0,
            0,
        );
        pos.make_move(mv);
        assert_eq!(pos.en_passant_square(), None);
    }

    // --- make_move: capture ---

    #[test]
    fn test_make_move_capture() {
        let mut pos = p("rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2");
        let mv = Move::new(
            Square::from_file_rank(4, 3),
            Square::from_file_rank(3, 4),
            0,
            0,
        );
        pos.make_move(mv);
        assert!(pos
            .pieces(Color::White, Piece::Pawn)
            .is_set(Square::from_file_rank(3, 4)));
        assert!(!pos
            .pieces(Color::Black, Piece::Pawn)
            .is_set(Square::from_file_rank(3, 4)));
    }

    // --- make_move: en passant ---

    #[test]
    fn test_make_move_en_passant() {
        // e5 pawn captures d6 en passant (black just pushed d7-d5)
        let mut pos = p("rnbqkbnr/ppp1pppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 3");
        let mv = Move::new(
            Square::from_file_rank(4, 4),
            Square::from_file_rank(3, 5),
            2,
            0,
        );
        pos.make_move(mv);
        assert!(pos
            .pieces(Color::White, Piece::Pawn)
            .is_set(Square::from_file_rank(3, 5)));
        assert!(!pos
            .pieces(Color::Black, Piece::Pawn)
            .is_set(Square::from_file_rank(3, 4)));
        assert!(!pos
            .pieces(Color::White, Piece::Pawn)
            .is_set(Square::from_file_rank(4, 4)));
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
        assert!(pos
            .pieces(Color::White, Piece::Queen)
            .is_set(Square::from_file_rank(0, 7)));
        assert!(!pos
            .pieces(Color::White, Piece::Pawn)
            .is_set(Square::from_file_rank(0, 6)));
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
        assert!(pos
            .pieces(Color::White, Piece::Knight)
            .is_set(Square::from_file_rank(0, 7)));
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
        assert!(pos
            .pieces(Color::White, Piece::Rook)
            .is_set(Square::from_file_rank(0, 7)));
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
        assert!(pos
            .pieces(Color::White, Piece::Bishop)
            .is_set(Square::from_file_rank(0, 7)));
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
        let mut pos = p("rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 5 2");
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
        let pos1 = p("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1");
        let pos2 = p("rnbqkbnr/pppppppp/8/8/3P4/8/PPP1PPPP/RNBQKBNR b KQkq d3 0 1");
        assert_ne!(pos1.hash(), pos2.hash());
    }

    #[test]
    fn test_zobrist_same_position_same_hash() {
        let pos1 = p("rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2");
        let pos2 = p("rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2");
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
