# Task 2: Implement board position and Zobrist hashing

## Trace
- **FR-IDs:** FR-01
- **Depends on:** task-1

## Files
- `src/board/mod.rs` — create
- `src/board/zobrist.rs` — create
- `src/lib.rs` — modify (add `board` module)

## Architecture

### Components
- `Position` struct: full board state using 12 bitboards (one per piece-color) plus side to move, castling rights, en passant square, halfmove clock, fullmove number, and Zobrist hash — new
- `Zobrist` module: random keys for incremental hashing — new

### Data Flow
FEN string → `Position::from_fen()` → populate bitboards + state fields + compute Zobrist hash
`Position::make_move()` → update bitboards, flip side, update castling/en passant, incrementally update Zobrist hash

## Contracts

### Internal Interfaces

**board/mod.rs — `Position`:**
- `Position::startpos() -> Position`: standard starting position
- `Position::from_fen(fen: &str) -> Result<Position, FenError>`: parse FEN string
- `Position::to_fen(&self) -> String`: serialize to FEN
- `Position::make_move(&mut self, mv: Move)`: apply move, update all state including Zobrist hash
  - Pre: move is pseudo-legal for this position
  - Post: position reflects the move, side to move flipped, Zobrist updated incrementally
- `Position::is_in_check(&self, color: Color) -> bool`: returns true if `color`'s king is attacked
  - Pre: attack generation available (uses bitboard ray/knight/pawn attacks)
  - Post: no side effects
- `Position::piece_at(&self, sq: Square) -> Option<(Color, Piece)>`: what's on this square
- `Position::pieces(&self, color: Color, piece: Piece) -> Bitboard`: bitboard for given piece/color
- `Position::occupancy(&self) -> Bitboard`: all occupied squares
- `Position::side_to_move(&self) -> Color`
- `Position::castling_rights(&self) -> CastlingRights`
- `Position::en_passant_square(&self) -> Option<Square>`
- `Position::hash(&self) -> u64`: current Zobrist hash
- `Position::is_draw_by_fifty(&self) -> bool`: halfmove clock >= 100

**board/zobrist.rs:**
- `zobrist::init()`: initialize random keys (called once at startup via `OnceLock`)
- `zobrist::piece_key(color: Color, piece: Piece, sq: Square) -> u64`
- `zobrist::castling_key(rights: CastlingRights) -> u64`
- `zobrist::en_passant_key(file: u8) -> u64`
- `zobrist::side_key() -> u64`: XOR when it's black to move
- Keys generated from a seeded PRNG for reproducibility

## Design Decisions

### Position state: 12 bitboards + metadata
- **Chosen:** Array of 12 bitboards `[Bitboard; 12]` indexed by `color * 6 + piece`, plus occupancy bitboards per color, side to move, castling rights, en passant square, halfmove/fullmove clocks, Zobrist hash
- **Rationale:** Direct indexing is fast, occupancy bitboards avoid recomputation. Struct size ~200 bytes, suitable for copy-make.
- **Rejected:** HashMap or sparse representation — unnecessary overhead for fixed 12-piece-type structure

### Move application: copy-make
- **Chosen:** Clone position before `make_move()`, no unmake needed
- **Rationale:** Clean ownership in Rust, no undo state tracking, Position is small enough to clone efficiently (~200 bytes). Simplifies search code significantly.
- **Rejected:** Make/unmake — error-prone undo logic, fights Rust's borrow checker

### Zobrist hashing: incremental updates
- **Chosen:** Full Zobrist with incremental XOR updates on make_move
- **Rationale:** O(1) hash update per move instead of O(n) full recomputation. Essential for transposition table performance.
- **Rejected:** Full recomputation — too slow for millions of positions in search

## Acceptance Criteria

### FR-01: Bitboard representation
- GIVEN any standard starting position
- WHEN the board is initialized
- THEN each piece type SHALL have a corresponding 64-bit bitboard with correct bit positions

- GIVEN a FEN string
- WHEN the board is loaded
- THEN the bitboard state SHALL match the FEN exactly

## Done Criteria
- [ ] `Position::startpos()` creates correct starting position (all 12 bitboards match)
- [ ] `Position::from_fen()` correctly parses at least 10 diverse FEN strings including edge cases
- [ ] `Position::to_fen()` round-trips correctly: `from_fen(to_fen(pos)) == pos`
- [ ] `Position::make_move()` correctly handles: quiet moves, captures, double pawn push, en passant, castling (all 4 types), promotions (all 4 types)
- [ ] Castling rights updated correctly when king/rook moves or rook is captured
- [ ] En passant square set on double pawn push, cleared otherwise
- [ ] Zobrist hash updated incrementally and matches full recomputation after each move
- [ ] `is_in_check()` correctly detects check from all piece types (using basic attack patterns — does not require magic bitboards; can use simple ray checks or be enhanced in task 3)
- [ ] Halfmove clock resets on pawn moves and captures
- [ ] Unit tests pass for all move types and FEN parsing
