/// Square index 0..63 using Little-Endian Rank-File (LERF): a1=0, h1=7, a2=8, h8=63.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Square(pub u8);

impl Square {
    pub fn from_file_rank(file: u8, rank: u8) -> Square {
        Square(rank * 8 + file)
    }

    pub fn file(self) -> u8 {
        self.0 % 8
    }

    pub fn rank(self) -> u8 {
        self.0 / 8
    }

    pub fn to_uci(self) -> String {
        let file = (b'a' + self.file()) as char;
        let rank = (b'1' + self.rank()) as char;
        format!("{}{}", file, rank)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn opposite(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

/// Packed u16 move: bits 0-5=from, 6-11=to, 12-13=flags, 14-15=promo piece.
/// Flags: 0=normal, 1=promotion, 2=en passant, 3=castling.
/// Promo: 0=Knight, 1=Bishop, 2=Rook, 3=Queen.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Move(pub u16);

impl Move {
    pub const NONE: Move = Move(0);

    pub fn new(from: Square, to: Square, flags: u8, promo: u8) -> Move {
        let val = (from.0 as u16)
            | ((to.0 as u16) << 6)
            | ((flags as u16) << 12)
            | ((promo as u16) << 14);
        Move(val)
    }

    pub fn from_sq(self) -> Square {
        Square((self.0 & 0x3F) as u8)
    }

    pub fn to_sq(self) -> Square {
        Square(((self.0 >> 6) & 0x3F) as u8)
    }

    pub fn flags(self) -> u8 {
        ((self.0 >> 12) & 0x3) as u8
    }

    pub fn promotion_piece(self) -> Piece {
        match (self.0 >> 14) & 0x3 {
            0 => Piece::Knight,
            1 => Piece::Bishop,
            2 => Piece::Rook,
            _ => Piece::Queen,
        }
    }

    pub fn is_promotion(self) -> bool {
        self.flags() == 1
    }

    pub fn is_en_passant(self) -> bool {
        self.flags() == 2
    }

    pub fn is_castling(self) -> bool {
        self.flags() == 3
    }

    pub fn to_uci(self) -> String {
        let from = self.from_sq().to_uci();
        let to = self.to_sq().to_uci();
        let mut s = format!("{}{}", from, to);
        if self.is_promotion() {
            let p = match self.promotion_piece() {
                Piece::Knight => 'n',
                Piece::Bishop => 'b',
                Piece::Rook => 'r',
                _ => 'q',
            };
            s.push(p);
        }
        s
    }
}

/// Bitmask for castling rights: WK=1, WQ=2, BK=4, BQ=8.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CastlingRights(pub u8);

impl CastlingRights {
    pub const WK: u8 = 1;
    pub const WQ: u8 = 2;
    pub const BK: u8 = 4;
    pub const BQ: u8 = 8;
    pub const ALL: u8 = 15;

    pub fn has(self, flag: u8) -> bool {
        self.0 & flag != 0
    }

    pub fn remove(&mut self, flag: u8) {
        self.0 &= !flag;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_square_from_file_rank() {
        assert_eq!(Square::from_file_rank(0, 0), Square(0)); // a1
        assert_eq!(Square::from_file_rank(7, 0), Square(7)); // h1
        assert_eq!(Square::from_file_rank(0, 7), Square(56)); // a8
        assert_eq!(Square::from_file_rank(7, 7), Square(63)); // h8
        assert_eq!(Square::from_file_rank(4, 1), Square(12)); // e2
    }

    #[test]
    fn test_square_file_rank() {
        let sq = Square(27); // d4: file=3, rank=3
        assert_eq!(sq.file(), 3);
        assert_eq!(sq.rank(), 3);

        let a1 = Square(0);
        assert_eq!(a1.file(), 0);
        assert_eq!(a1.rank(), 0);

        let h8 = Square(63);
        assert_eq!(h8.file(), 7);
        assert_eq!(h8.rank(), 7);
    }

    #[test]
    fn test_color_opposite() {
        assert_eq!(Color::White.opposite(), Color::Black);
        assert_eq!(Color::Black.opposite(), Color::White);
    }

    #[test]
    fn test_move_none() {
        assert_eq!(Move::NONE, Move(0));
    }

    #[test]
    fn test_move_normal() {
        let from = Square(8); // a2
        let to = Square(16); // a3
        let m = Move::new(from, to, 0, 0);
        assert_eq!(m.from_sq(), from);
        assert_eq!(m.to_sq(), to);
        assert_eq!(m.flags(), 0);
        assert!(!m.is_promotion());
        assert!(!m.is_en_passant());
        assert!(!m.is_castling());
    }

    #[test]
    fn test_move_promotion() {
        let from = Square(48); // a7
        let to = Square(56); // a8
        let m = Move::new(from, to, 1, 3); // flag=promotion, promo=Queen
        assert_eq!(m.from_sq(), from);
        assert_eq!(m.to_sq(), to);
        assert_eq!(m.flags(), 1);
        assert!(m.is_promotion());
        assert_eq!(m.promotion_piece(), Piece::Queen);
    }

    #[test]
    fn test_move_promotion_pieces() {
        let from = Square(48);
        let to = Square(56);
        assert_eq!(Move::new(from, to, 1, 0).promotion_piece(), Piece::Knight);
        assert_eq!(Move::new(from, to, 1, 1).promotion_piece(), Piece::Bishop);
        assert_eq!(Move::new(from, to, 1, 2).promotion_piece(), Piece::Rook);
        assert_eq!(Move::new(from, to, 1, 3).promotion_piece(), Piece::Queen);
    }

    #[test]
    fn test_move_en_passant() {
        let m = Move::new(Square(32), Square(41), 2, 0);
        assert!(m.is_en_passant());
        assert!(!m.is_promotion());
        assert!(!m.is_castling());
    }

    #[test]
    fn test_move_castling() {
        let m = Move::new(Square(4), Square(6), 3, 0);
        assert!(m.is_castling());
        assert!(!m.is_promotion());
        assert!(!m.is_en_passant());
    }

    #[test]
    fn test_castling_rights_all() {
        let rights = CastlingRights(CastlingRights::ALL);
        assert!(rights.has(CastlingRights::WK));
        assert!(rights.has(CastlingRights::WQ));
        assert!(rights.has(CastlingRights::BK));
        assert!(rights.has(CastlingRights::BQ));
    }

    #[test]
    fn test_castling_rights_none() {
        let rights = CastlingRights(0);
        assert!(!rights.has(CastlingRights::WK));
        assert!(!rights.has(CastlingRights::WQ));
        assert!(!rights.has(CastlingRights::BK));
        assert!(!rights.has(CastlingRights::BQ));
    }

    #[test]
    fn test_castling_rights_remove() {
        let mut rights = CastlingRights(CastlingRights::ALL);
        rights.remove(CastlingRights::WK);
        assert!(!rights.has(CastlingRights::WK));
        assert!(rights.has(CastlingRights::WQ));
        assert!(rights.has(CastlingRights::BK));
        assert!(rights.has(CastlingRights::BQ));
    }
}
