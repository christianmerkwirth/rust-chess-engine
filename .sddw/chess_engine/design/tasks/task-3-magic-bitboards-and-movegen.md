# Task 3: Implement magic bitboards and move generation

## Trace
- **FR-IDs:** FR-02
- **Depends on:** task-1, task-2

## Files
- `src/movegen/mod.rs` — create
- `src/movegen/magics.rs` — create
- `src/lib.rs` — modify (add `movegen` module)

## Architecture

### Components
- `magics` module: magic bitboard tables for sliding piece attacks (bishop, rook), runtime-initialized via `OnceLock` — new
- `movegen` module: pseudo-legal move generation for all piece types, legality filtering, perft — new

### Data Flow
Engine startup → `magics::init()` (compute magic tables for bishop/rook attacks, ~10ms)
Position → `movegen::generate_moves()` → pseudo-legal moves → filter via make_move + is_in_check → legal `MoveList`
Position + depth → `movegen::perft()` → recursive node count for correctness testing

## Contracts

### Internal Interfaces

**movegen/magics.rs:**
- `magics::init()`: compute and store magic bitboard tables (called once via `OnceLock`)
- `magics::bishop_attacks(sq: Square, occupancy: Bitboard) -> Bitboard`: bishop attack bitboard
- `magics::rook_attacks(sq: Square, occupancy: Bitboard) -> Bitboard`: rook attack bitboard
- `magics::queen_attacks(sq: Square, occupancy: Bitboard) -> Bitboard`: union of bishop + rook attacks
- Knight and king attack tables: pre-computed `[Bitboard; 64]` arrays (not magic, just lookup)
- Pawn attack tables: `PAWN_ATTACKS[Color][Square]` — pre-computed `[[Bitboard; 64]; 2]`

**movegen/mod.rs:**
- `MoveList` struct: stack-allocated move list (e.g., `ArrayVec<Move, 256>` or fixed array + count)
- `generate_moves(pos: &Position) -> MoveList`: all legal moves for the side to move
  - Pre: magics initialized
  - Post: returns only legal moves (pseudo-legal filtered by legality check)
- `generate_captures(pos: &Position) -> MoveList`: legal capture moves only (for quiescence search)
- `is_square_attacked(pos: &Position, sq: Square, by_color: Color) -> bool`: is this square attacked
  - Used by: legality check (is king in check after move), castling validation
- `perft(pos: &Position, depth: u32) -> u64`: recursive node count for move generation verification
  - Pre: depth >= 1
  - Post: returns total leaf node count

**Move generation covers all special cases:**
- Pawn: single push, double push (from rank 2/7), captures, en passant, promotions (Q/R/B/N)
- Knight: standard L-shape jumps
- Bishop/Rook/Queen: magic bitboard lookup, masked to legal targets
- King: standard moves + castling (validate: king not in check, transit squares not attacked, no pieces between, rights exist)
- All moves filtered: make move on clone, check if own king is in check → discard if yes

## Design Decisions

### Move generation: pseudo-legal + legality filter
- **Chosen:** Generate all pseudo-legal moves, then filter by making on a clone and checking king safety
- **Rationale:** Simpler to implement and debug. The copy-make approach makes the filter cheap. Correctness is easy to verify via perft.
- **Rejected:** Fully legal generation with pin/check masks — more complex, error-prone, marginal performance gain that matters less at ~1800 ELO target

### Magic number generation: runtime with fixed seeds
- **Chosen:** Generate magic numbers at runtime using a seeded PRNG, store in `OnceLock`-guarded statics
- **Rationale:** ~10ms startup cost is negligible. Avoids bloating the binary with large const tables. Reproducible via fixed seed.
- **Rejected:** Compile-time const generation — inflates compile times significantly, complex `const fn` requirements

### MoveList: stack-allocated fixed array
- **Chosen:** `[Move; 256]` array with a length counter (max legal moves in any position is 218)
- **Rationale:** No heap allocation in the hot path. 256 * 2 bytes = 512 bytes on the stack per call, well within stack limits even at depth 100+.
- **Rejected:** `Vec<Move>` — heap allocation per node is expensive in search; `ArrayVec` — adds a dependency for something trivial to implement

## Acceptance Criteria

### FR-02: Legal move generation
- GIVEN the standard starting position
- WHEN a perft test is run to depth 6
- THEN the node count SHALL match the known correct value (119,060,324)

- GIVEN a position with castling rights
- WHEN the king or rook has moved or squares are attacked
- THEN castling SHALL only be generated when legal

- GIVEN a position with an en passant opportunity
- WHEN the en passant capture would leave the king in check
- THEN the move SHALL NOT be generated

- GIVEN a position with a pawn on the 7th/2nd rank
- WHEN the pawn advances or captures
- THEN all four promotion types (Q, R, B, N) SHALL be generated

## Done Criteria
- [ ] Magic bitboard tables initialize correctly at runtime (~10ms)
- [ ] `bishop_attacks()` and `rook_attacks()` return correct attack bitboards for all 64 squares with various occupancies
- [ ] Knight and king attack lookup tables are correct for all 64 squares
- [ ] Pawn attack tables correct for both colors and all squares
- [ ] `generate_moves()` produces only legal moves (no illegal moves pass through)
- [ ] Perft results match known values for standard starting position (depths 1-6)
- [ ] Perft results match for at least 5 additional test positions (Kiwipete, position 3-5 from CPW perft suite)
- [ ] Castling only generated when all conditions met (rights, no pieces between, king not in check, transit squares not attacked)
- [ ] En passant correctly generated and correctly rejected when it would expose king to check
- [ ] All four promotion types generated for pawn advances and captures on promotion rank
- [ ] `generate_captures()` returns only capture moves (for quiescence)
- [ ] `is_square_attacked()` works correctly for all piece types
- [ ] Unit tests pass for each piece type's move generation individually
