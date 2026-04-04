use std::sync::OnceLock;

use crate::types::{CastlingRights, Color, Piece, Square};

struct ZobristKeys {
    pieces: [[u64; 64]; 12],
    castling: [u64; 16],
    en_passant: [u64; 8],
    side: u64,
}

static KEYS: OnceLock<ZobristKeys> = OnceLock::new();

fn init_keys() -> ZobristKeys {
    todo!()
}

fn keys() -> &'static ZobristKeys {
    KEYS.get_or_init(init_keys)
}

pub fn piece_key(_color: Color, _piece: Piece, _sq: Square) -> u64 {
    todo!()
}

pub fn castling_key(_rights: CastlingRights) -> u64 {
    todo!()
}

pub fn en_passant_key(_file: u8) -> u64 {
    todo!()
}

pub fn side_key() -> u64 {
    todo!()
}
