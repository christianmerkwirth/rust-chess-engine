use std::sync::OnceLock;

use crate::types::{CastlingRights, Color, Piece, Square};

struct ZobristKeys {
    pieces: [[u64; 64]; 12],
    castling: [u64; 16],
    en_passant: [u64; 8],
    side: u64,
}

static KEYS: OnceLock<ZobristKeys> = OnceLock::new();

fn xorshift64(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

fn init_keys() -> ZobristKeys {
    let mut s: u64 = 0x246F_EAEF_5239_D787;
    let mut pieces = [[0u64; 64]; 12];
    for pi in pieces.iter_mut() {
        for k in pi.iter_mut() {
            *k = xorshift64(&mut s);
        }
    }
    let mut castling = [0u64; 16];
    for k in castling.iter_mut() {
        *k = xorshift64(&mut s);
    }
    let mut en_passant = [0u64; 8];
    for k in en_passant.iter_mut() {
        *k = xorshift64(&mut s);
    }
    let side = xorshift64(&mut s);
    ZobristKeys {
        pieces,
        castling,
        en_passant,
        side,
    }
}

fn keys() -> &'static ZobristKeys {
    KEYS.get_or_init(init_keys)
}

pub fn piece_key(color: Color, piece: Piece, sq: Square) -> u64 {
    keys().pieces[color as usize * 6 + piece as usize][sq.0 as usize]
}

pub fn castling_key(rights: CastlingRights) -> u64 {
    keys().castling[rights.0 as usize]
}

pub fn en_passant_key(file: u8) -> u64 {
    keys().en_passant[file as usize]
}

pub fn side_key() -> u64 {
    keys().side
}
