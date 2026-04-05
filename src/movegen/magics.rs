use crate::bitboard::Bitboard;
use crate::types::{Color, Square};
use std::sync::OnceLock;

/// Prng for magic number generation (deterministic with fixed seed).
struct Prng(u64);

impl Prng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn sparse_rand(&mut self) -> u64 {
        self.next() & self.next() & self.next()
    }
}

#[derive(Copy, Clone)]
pub struct Magic {
    pub mask: Bitboard,
    pub magic: u64,
    pub shift: u8,
    pub offset: usize,
}

static KING_ATTACKS: OnceLock<[Bitboard; 64]> = OnceLock::new();
static KNIGHT_ATTACKS: OnceLock<[Bitboard; 64]> = OnceLock::new();
static PAWN_ATTACKS: OnceLock<[[Bitboard; 64]; 2]> = OnceLock::new();

static BISHOP_MAGICS: OnceLock<[Magic; 64]> = OnceLock::new();
static ROOK_MAGICS: OnceLock<[Magic; 64]> = OnceLock::new();

static ATTACK_TABLE: OnceLock<Vec<Bitboard>> = OnceLock::new();

pub fn init() {
    _ = KING_ATTACKS.get_or_init(init_king_attacks);
    _ = KNIGHT_ATTACKS.get_or_init(init_knight_attacks);
    _ = PAWN_ATTACKS.get_or_init(init_pawn_attacks);

    let mut prng = Prng(123456789);

    // Initialize magics
    let mut offset = 0;
    let mut bishop_magics = [Magic {
        mask: Bitboard::empty(),
        magic: 0,
        shift: 0,
        offset: 0,
    }; 64];
    let mut rook_magics = [Magic {
        mask: Bitboard::empty(),
        magic: 0,
        shift: 0,
        offset: 0,
    }; 64];
    let mut table = Vec::with_capacity(107648);

    for sq in 0..64 {
        let square = Square(sq);
        let mask = bishop_mask(square);
        let shift = 64 - mask.count() as u8;
        let permutations = 1 << mask.count();

        let mut occs = Vec::with_capacity(permutations);
        let mut attacks = Vec::with_capacity(permutations);
        for i in 0..permutations {
            let occ = index_to_occupancy(i as u32, mask);
            occs.push(occ);
            attacks.push(bishop_attacks_slow(square, occ));
        }

        let magic_num = loop {
            let m = prng.sparse_rand();
            if (Bitboard(mask.0.wrapping_mul(m)).0 >> 56).count_ones() < 6 {
                continue;
            }

            let mut test_table = vec![Bitboard::empty(); permutations];
            let mut ok = true;
            for i in 0..permutations {
                let idx = ((occs[i].0.wrapping_mul(m)) >> shift) as usize;
                if test_table[idx].is_empty() || test_table[idx] == attacks[i] {
                    test_table[idx] = attacks[i];
                } else {
                    ok = false;
                    break;
                }
            }
            if ok {
                break m;
            }
        };

        bishop_magics[sq as usize] = Magic {
            mask,
            magic: magic_num,
            shift,
            offset,
        };
        for i in 0..permutations {
            let idx = ((occs[i].0.wrapping_mul(magic_num)) >> shift) as usize;
            while table.len() <= offset + idx {
                table.push(Bitboard::empty());
            }
            table[offset + idx] = attacks[i];
        }
        offset += permutations;
    }

    for sq in 0..64 {
        let square = Square(sq);
        let mask = rook_mask(square);
        let shift = 64 - mask.count() as u8;
        let permutations = 1 << mask.count();

        let mut occs = Vec::with_capacity(permutations);
        let mut attacks = Vec::with_capacity(permutations);
        for i in 0..permutations {
            let occ = index_to_occupancy(i as u32, mask);
            occs.push(occ);
            attacks.push(rook_attacks_slow(square, occ));
        }

        let magic_num = loop {
            let m = prng.sparse_rand();
            if (Bitboard(mask.0.wrapping_mul(m)).0 >> 56).count_ones() < 6 {
                continue;
            }

            let mut test_table = vec![Bitboard::empty(); permutations];
            let mut ok = true;
            for i in 0..permutations {
                let idx = ((occs[i].0.wrapping_mul(m)) >> shift) as usize;
                if test_table[idx].is_empty() || test_table[idx] == attacks[i] {
                    test_table[idx] = attacks[i];
                } else {
                    ok = false;
                    break;
                }
            }
            if ok {
                break m;
            }
        };

        rook_magics[sq as usize] = Magic {
            mask,
            magic: magic_num,
            shift,
            offset,
        };
        for i in 0..permutations {
            let idx = ((occs[i].0.wrapping_mul(magic_num)) >> shift) as usize;
            while table.len() <= offset + idx {
                table.push(Bitboard::empty());
            }
            table[offset + idx] = attacks[i];
        }
        offset += permutations;
    }

    _ = BISHOP_MAGICS.get_or_init(|| bishop_magics);
    _ = ROOK_MAGICS.get_or_init(|| rook_magics);
    _ = ATTACK_TABLE.get_or_init(|| table);
}

pub fn bishop_attacks(sq: Square, occupancy: Bitboard) -> Bitboard {
    let m = &BISHOP_MAGICS.get().expect("magics not initialized")[sq.0 as usize];
    let occ = occupancy & m.mask;
    let idx = ((occ.0.wrapping_mul(m.magic)) >> m.shift) as usize;
    ATTACK_TABLE.get().expect("attack table not initialized")[m.offset + idx]
}

pub fn rook_attacks(sq: Square, occupancy: Bitboard) -> Bitboard {
    let m = &ROOK_MAGICS.get().expect("magics not initialized")[sq.0 as usize];
    let occ = occupancy & m.mask;
    let idx = ((occ.0.wrapping_mul(m.magic)) >> m.shift) as usize;
    ATTACK_TABLE.get().expect("attack table not initialized")[m.offset + idx]
}

pub fn queen_attacks(sq: Square, occupancy: Bitboard) -> Bitboard {
    bishop_attacks(sq, occupancy) | rook_attacks(sq, occupancy)
}

pub fn knight_attacks(sq: Square) -> Bitboard {
    KNIGHT_ATTACKS.get().expect("magics not initialized")[sq.0 as usize]
}

pub fn king_attacks(sq: Square) -> Bitboard {
    KING_ATTACKS.get().expect("magics not initialized")[sq.0 as usize]
}

pub fn pawn_attacks(color: Color, sq: Square) -> Bitboard {
    PAWN_ATTACKS.get().expect("magics not initialized")[color as usize][sq.0 as usize]
}

// --- Internal initializers ---

fn init_king_attacks() -> [Bitboard; 64] {
    let mut table = [Bitboard::empty(); 64];
    for sq in 0..64 {
        let mut bb = Bitboard::empty();
        let s = Square(sq);
        let f = s.file() as i32;
        let r = s.rank() as i32;
        for df in -1..=1 {
            for dr in -1..=1 {
                if df == 0 && dr == 0 {
                    continue;
                }
                let nf = f + df;
                let nr = r + dr;
                if (0..8).contains(&nf) && (0..8).contains(&nr) {
                    bb.set(Square::from_file_rank(nf as u8, nr as u8));
                }
            }
        }
        table[sq as usize] = bb;
    }
    table
}

fn init_knight_attacks() -> [Bitboard; 64] {
    let mut table = [Bitboard::empty(); 64];
    for sq in 0..64 {
        let mut bb = Bitboard::empty();
        let s = Square(sq);
        let f = s.file() as i32;
        let r = s.rank() as i32;
        for &(df, dr) in &[
            (1, 2),
            (1, -2),
            (-1, 2),
            (-1, -2),
            (2, 1),
            (2, -1),
            (-2, 1),
            (-2, -1),
        ] {
            let nf = f + df;
            let nr = r + dr;
            if (0..8).contains(&nf) && (0..8).contains(&nr) {
                bb.set(Square::from_file_rank(nf as u8, nr as u8));
            }
        }
        table[sq as usize] = bb;
    }
    table
}

fn init_pawn_attacks() -> [[Bitboard; 64]; 2] {
    let mut table = [[Bitboard::empty(); 64]; 2];
    for sq in 0..64 {
        let s = Square(sq);
        let f = s.file() as i32;
        let r = s.rank() as i32;

        // White
        if r < 7 {
            for df in &[-1, 1] {
                let nf = f + df;
                if (0..8).contains(&nf) {
                    table[Color::White as usize][sq as usize]
                        .set(Square::from_file_rank(nf as u8, (r + 1) as u8));
                }
            }
        }

        // Black
        if r > 0 {
            for df in &[-1, 1] {
                let nf = f + df;
                if (0..8).contains(&nf) {
                    table[Color::Black as usize][sq as usize]
                        .set(Square::from_file_rank(nf as u8, (r - 1) as u8));
                }
            }
        }
    }
    table
}

fn bishop_mask(sq: Square) -> Bitboard {
    let mut bb = Bitboard::empty();
    let r = sq.rank() as i32;
    let f = sq.file() as i32;
    for (dr, df) in &[(1, 1), (1, -1), (-1, 1), (-1, -1)] {
        let mut nr = r + dr;
        let mut nf = f + df;
        while nr > 0 && nr < 7 && nf > 0 && nf < 7 {
            bb.set(Square::from_file_rank(nf as u8, nr as u8));
            nr += dr;
            nf += df;
        }
    }
    bb
}

fn rook_mask(sq: Square) -> Bitboard {
    let mut bb = Bitboard::empty();
    let r = sq.rank() as i32;
    let f = sq.file() as i32;
    // North
    let mut nr = r + 1;
    while nr < 7 {
        bb.set(Square::from_file_rank(f as u8, nr as u8));
        nr += 1;
    }
    // South
    let mut nr = r - 1;
    while nr > 0 {
        bb.set(Square::from_file_rank(f as u8, nr as u8));
        nr -= 1;
    }
    // East
    let mut nf = f + 1;
    while nf < 7 {
        bb.set(Square::from_file_rank(nf as u8, r as u8));
        nf += 1;
    }
    // West
    let mut nf = f - 1;
    while nf > 0 {
        bb.set(Square::from_file_rank(nf as u8, r as u8));
        nf -= 1;
    }
    bb
}

fn bishop_attacks_slow(sq: Square, occ: Bitboard) -> Bitboard {
    let mut bb = Bitboard::empty();
    let r = sq.rank() as i32;
    let f = sq.file() as i32;
    for (dr, df) in &[(1, 1), (1, -1), (-1, 1), (-1, -1)] {
        let mut nr = r + dr;
        let mut nf = f + df;
        while (0..8).contains(&nr) && (0..8).contains(&nf) {
            let target = Square::from_file_rank(nf as u8, nr as u8);
            bb.set(target);
            if occ.is_set(target) {
                break;
            }
            nr += dr;
            nf += df;
        }
    }
    bb
}

fn rook_attacks_slow(sq: Square, occ: Bitboard) -> Bitboard {
    let mut bb = Bitboard::empty();
    let r = sq.rank() as i32;
    let f = sq.file() as i32;
    for (dr, df) in &[(1, 0), (-1, 0), (0, 1), (0, -1)] {
        let mut nr = r + dr;
        let mut nf = f + df;
        while (0..8).contains(&nr) && (0..8).contains(&nf) {
            let target = Square::from_file_rank(nf as u8, nr as u8);
            bb.set(target);
            if occ.is_set(target) {
                break;
            }
            nr += dr;
            nf += df;
        }
    }
    bb
}

fn index_to_occupancy(index: u32, mask: Bitboard) -> Bitboard {
    let mut occ = Bitboard::empty();
    let mut mask = mask;
    let mut i = 0;
    while !mask.is_empty() {
        let sq = mask.pop_lsb();
        if (index & (1 << i)) != 0 {
            occ.set(sq);
        }
        i += 1;
    }
    occ
}
