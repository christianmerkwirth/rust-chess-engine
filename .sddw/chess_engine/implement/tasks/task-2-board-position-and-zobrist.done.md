# Task 2 Completion: Implement board position and Zobrist hashing

## Summary
Created `src/board/zobrist.rs` (seeded xorshift64 PRNG via `OnceLock`, keys for all 12 piece/square combinations, 16 castling combinations, 8 en-passant files, side-to-move) and `src/board/mod.rs` (`Position` struct with 12 piece bitboards + 2 color occupancy bitboards, full FEN parsing/serialisation, `make_move` with incremental Zobrist updates, and simple ray-based `is_in_check`). Added `pub mod board` to `src/lib.rs`. All 54 new tests pass (77 total).

## Commits
- `8d5a1c5` test(chess-engine): add failing tests for position and Zobrist (FR-01)
- `4199e5d` feat(chess-engine): implement board position and Zobrist hashing (FR-01)

## Deviations
- **Rule 2: Missing Critical** — added `compute_hash()` as a public method for test verification of incremental Zobrist updates; not in the original interface spec but needed to satisfy the "hash matches full recomputation" done criterion.

## Difficulties
- Rust field/method name collision: struct field named `pieces` conflicted with the `pieces(&self, color, piece)` method — resolved by renaming the internal field to `piece_bb`.
- Temporary-value lifetime error in tests: `pos.to_fen().split(' ').collect()` — fixed by binding the `String` to a `let` before splitting.

## Notes
- `is_in_check` uses simple ray tracing (O(rays × depth)); will be superseded by magic-bitboard attack tables in task 3 without any API change.
- Copy-make semantics: callers should `clone()` before `make_move()` when they need to restore the prior position (e.g., in search).
- Castling rights are updated via two rules: king-move clears both rights for its color; any move touching a rook's origin square (a1/h1/a8/h8) clears the corresponding right, covering both rook moves and captures of rooks on their starting squares.
