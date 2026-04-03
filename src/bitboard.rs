use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, Shr};

use crate::types::Square;

/// A 64-bit board mask. Bit N corresponds to Square(N) in LERF layout.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Bitboard(pub u64);

impl Bitboard {
    pub const FILE_A: Bitboard = Bitboard(0);
    pub const FILE_B: Bitboard = Bitboard(0);
    pub const FILE_C: Bitboard = Bitboard(0);
    pub const FILE_D: Bitboard = Bitboard(0);
    pub const FILE_E: Bitboard = Bitboard(0);
    pub const FILE_F: Bitboard = Bitboard(0);
    pub const FILE_G: Bitboard = Bitboard(0);
    pub const FILE_H: Bitboard = Bitboard(0);

    pub const RANK_1: Bitboard = Bitboard(0);
    pub const RANK_2: Bitboard = Bitboard(0);
    pub const RANK_3: Bitboard = Bitboard(0);
    pub const RANK_4: Bitboard = Bitboard(0);
    pub const RANK_5: Bitboard = Bitboard(0);
    pub const RANK_6: Bitboard = Bitboard(0);
    pub const RANK_7: Bitboard = Bitboard(0);
    pub const RANK_8: Bitboard = Bitboard(0);

    pub fn empty() -> Bitboard {
        todo!()
    }

    pub fn full() -> Bitboard {
        todo!()
    }

    pub fn from_square(_sq: Square) -> Bitboard {
        todo!()
    }

    pub fn is_set(self, _sq: Square) -> bool {
        todo!()
    }

    pub fn set(&mut self, _sq: Square) {
        todo!()
    }

    pub fn clear(&mut self, _sq: Square) {
        todo!()
    }

    pub fn count(self) -> u32 {
        todo!()
    }

    pub fn is_empty(self) -> bool {
        todo!()
    }

    pub fn lsb(self) -> Square {
        todo!()
    }

    pub fn pop_lsb(&mut self) -> Square {
        todo!()
    }
}

impl Iterator for Bitboard {
    type Item = Square;

    fn next(&mut self) -> Option<Square> {
        todo!()
    }
}

impl BitAnd for Bitboard {
    type Output = Bitboard;
    fn bitand(self, rhs: Bitboard) -> Bitboard {
        todo!()
    }
}

impl BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, rhs: Bitboard) {
        *self = *self & rhs;
    }
}

impl BitOr for Bitboard {
    type Output = Bitboard;
    fn bitor(self, rhs: Bitboard) -> Bitboard {
        todo!()
    }
}

impl BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Bitboard) {
        *self = *self | rhs;
    }
}

impl BitXor for Bitboard {
    type Output = Bitboard;
    fn bitxor(self, rhs: Bitboard) -> Bitboard {
        todo!()
    }
}

impl BitXorAssign for Bitboard {
    fn bitxor_assign(&mut self, rhs: Bitboard) {
        *self = *self ^ rhs;
    }
}

impl Not for Bitboard {
    type Output = Bitboard;
    fn not(self) -> Bitboard {
        todo!()
    }
}

impl Shl<u32> for Bitboard {
    type Output = Bitboard;
    fn shl(self, rhs: u32) -> Bitboard {
        todo!()
    }
}

impl Shr<u32> for Bitboard {
    type Output = Bitboard;
    fn shr(self, rhs: u32) -> Bitboard {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_and_full() {
        assert_eq!(Bitboard::empty(), Bitboard(0));
        assert_eq!(Bitboard::full(), Bitboard(!0u64));
        assert!(Bitboard::empty().is_empty());
        assert!(!Bitboard::full().is_empty());
    }

    #[test]
    fn test_from_square() {
        assert_eq!(Bitboard::from_square(Square(0)), Bitboard(1));
        assert_eq!(Bitboard::from_square(Square(1)), Bitboard(2));
        assert_eq!(Bitboard::from_square(Square(63)), Bitboard(1u64 << 63));
    }

    #[test]
    fn test_set_clear_is_set() {
        let mut bb = Bitboard::empty();
        assert!(!bb.is_set(Square(5)));
        bb.set(Square(5));
        assert!(bb.is_set(Square(5)));
        bb.clear(Square(5));
        assert!(!bb.is_set(Square(5)));
    }

    #[test]
    fn test_count() {
        assert_eq!(Bitboard::empty().count(), 0);
        assert_eq!(Bitboard::full().count(), 64);
        assert_eq!(Bitboard::from_square(Square(0)).count(), 1);

        let mut bb = Bitboard::empty();
        bb.set(Square(0));
        bb.set(Square(7));
        bb.set(Square(63));
        assert_eq!(bb.count(), 3);
    }

    #[test]
    fn test_lsb() {
        assert_eq!(Bitboard(0b1100).lsb(), Square(2));
        assert_eq!(Bitboard::from_square(Square(7)).lsb(), Square(7));
    }

    #[test]
    fn test_pop_lsb() {
        let mut bb = Bitboard(0b1100);
        assert_eq!(bb.pop_lsb(), Square(2));
        assert_eq!(bb, Bitboard(0b1000));
        assert_eq!(bb.pop_lsb(), Square(3));
        assert!(bb.is_empty());
    }

    #[test]
    fn test_bitwise_ops() {
        let a = Bitboard(0b1010);
        let b = Bitboard(0b1100);
        assert_eq!(a & b, Bitboard(0b1000));
        assert_eq!(a | b, Bitboard(0b1110));
        assert_eq!(a ^ b, Bitboard(0b0110));
        assert_eq!(!a, Bitboard(!0b1010u64));
    }

    #[test]
    fn test_shifts() {
        let bb = Bitboard(1);
        assert_eq!(bb << 1, Bitboard(2));
        assert_eq!(bb << 8, Bitboard(256));

        let bb = Bitboard(256);
        assert_eq!(bb >> 8, Bitboard(1));
    }

    #[test]
    fn test_iterator() {
        let mut bb = Bitboard::empty();
        bb.set(Square(0));
        bb.set(Square(5));
        bb.set(Square(63));

        let squares: Vec<Square> = bb.collect();
        assert_eq!(squares, vec![Square(0), Square(5), Square(63)]);
    }

    #[test]
    fn test_file_constants() {
        let expected_a: u64 = (0..8u32).map(|r| 1u64 << (r * 8)).sum();
        assert_eq!(Bitboard::FILE_A, Bitboard(expected_a));

        let expected_h: u64 = (0..8u32).map(|r| 1u64 << (r * 8 + 7)).sum();
        assert_eq!(Bitboard::FILE_H, Bitboard(expected_h));
    }

    #[test]
    fn test_rank_constants() {
        assert_eq!(Bitboard::RANK_1, Bitboard(0x00000000000000FF));
        assert_eq!(Bitboard::RANK_8, Bitboard(0xFF00000000000000));
    }
}
