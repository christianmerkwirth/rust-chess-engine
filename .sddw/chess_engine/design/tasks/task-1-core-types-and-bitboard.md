# Task 1: Create core types and bitboard module

## Trace
- **FR-IDs:** FR-01
- **Depends on:** none

## Files
- `src/types.rs` — create
- `src/bitboard.rs` — create
- `src/lib.rs` — create
- `Cargo.toml` — create

## Architecture

### Components
- `types` module: foundational chess types shared across all modules — new
- `bitboard` module: 64-bit board representation with bit manipulation operations — new

### Data Flow
`Cargo.toml` defines the crate → `lib.rs` re-exports modules → `types.rs` provides `Square`, `Piece`, `Color`, `Move`, `CastlingRights` → `bitboard.rs` provides `Bitboard` newtype wrapping `u64`

## Contracts

### Internal Interfaces

**types.rs:**
- `Square` enum or newtype (0..63, a1=0, h8=63): maps file/rank to bit index
  - `Square::from_file_rank(file: u8, rank: u8) -> Square`
  - `Square::file(self) -> u8` / `Square::rank(self) -> u8`
- `Color` enum: `White`, `Black`
  - `Color::opposite(self) -> Color`
- `Piece` enum: `Pawn`, `Knight`, `Bishop`, `Rook`, `Queen`, `King`
- `Move` newtype (`u16`): packed representation
  - Bits 0-5: from square
  - Bits 6-11: to square
  - Bits 12-13: flags (0=normal, 1=promotion, 2=en passant, 3=castling)
  - Bits 14-15: promotion piece (0=Knight, 1=Bishop, 2=Rook, 3=Queen) — only meaningful when flag=1
  - `Move::new(from: Square, to: Square, flags: u8, promo: u8) -> Move`
  - `Move::from_sq(self) -> Square` / `Move::to_sq(self) -> Square`
  - `Move::flags(self) -> u8` / `Move::promotion_piece(self) -> Piece`
  - `Move::is_promotion(self) -> bool` / `Move::is_en_passant(self) -> bool` / `Move::is_castling(self) -> bool`
  - `Move::NONE` — sentinel null move (e.g., 0)
- `CastlingRights` newtype (`u8`): bitmask for WK, WQ, BK, BQ
  - `CastlingRights::has(self, flag: u8) -> bool`
  - `CastlingRights::remove(&mut self, flag: u8)`
  - Constants: `WK = 1`, `WQ = 2`, `BK = 4`, `BQ = 8`, `ALL = 15`

**bitboard.rs:**
- `Bitboard` newtype (`u64`)
  - `Bitboard::empty() -> Bitboard` — all zeros
  - `Bitboard::full() -> Bitboard` — all ones
  - `Bitboard::from_square(sq: Square) -> Bitboard` — single bit set
  - `Bitboard::is_set(self, sq: Square) -> bool`
  - `Bitboard::set(&mut self, sq: Square)`
  - `Bitboard::clear(&mut self, sq: Square)`
  - `Bitboard::count(self) -> u32` — popcount
  - `Bitboard::is_empty(self) -> bool`
  - `Bitboard::lsb(self) -> Square` — least significant bit (for iteration)
  - `Bitboard::pop_lsb(&mut self) -> Square` — pop and return LSB
  - Implement `BitAnd`, `BitOr`, `BitXor`, `Not`, `Shl`, `Shr` for `Bitboard`
  - Implement `Iterator` for `Bitboard` (iterates over set squares)
  - Constants: `FILE_A..FILE_H`, `RANK_1..RANK_8` as pre-computed masks

## Design Decisions

### Move encoding: packed u16
- **Chosen:** Packed `u16` with 6+6+2+2 bit layout
- **Rationale:** Cache-friendly, matches Stockfish encoding, fits in registers. 4-bit flag space encodes all special move types (promotion, en passant, castling) without needing separate fields.
- **Rejected:** Struct with named fields — wastes memory in move lists, worse cache behavior in search

### Square indexing: Little-Endian Rank-File (LERF)
- **Chosen:** a1=0, b1=1, ..., h1=7, a2=8, ..., h8=63
- **Rationale:** Standard in chess programming, matches bit shifting for rank/file operations. North = +8, South = -8, East = +1, West = -1.
- **Rejected:** Big-endian (a8=0) — less common, confusing shift directions

## Acceptance Criteria

### FR-01: Bitboard representation
- GIVEN any standard starting position
- WHEN the board is initialized
- THEN each piece type SHALL have a corresponding 64-bit bitboard with correct bit positions

## Done Criteria
- [ ] `Cargo.toml` exists with Rust 2021 edition, binary target, no external dependencies yet
- [ ] `Square` type supports conversion to/from file+rank and to/from index 0..63
- [ ] `Move` type correctly packs/unpacks from/to/flags/promotion in a `u16`
- [ ] `Bitboard` supports all listed operations (`set`, `clear`, `count`, `lsb`, `pop_lsb`, bitwise ops)
- [ ] `Bitboard` constants for files A-H and ranks 1-8 are correct
- [ ] `Bitboard` iterator yields all set squares in order
- [ ] `CastlingRights` bitmask operations work correctly
- [ ] All types implement `Copy`, `Clone`, `Debug`, `PartialEq`, `Eq`
- [ ] Unit tests pass for all type conversions and bitboard operations
