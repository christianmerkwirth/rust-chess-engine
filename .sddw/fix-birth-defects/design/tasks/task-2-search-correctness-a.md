# Task 2: Search Correctness - Part A

Implement three-fold repetition and insufficient material detection to handle basic draws correctly. Add perft depth 6 to verify move generation.

## 1. Requirements

- FR-05: Three-fold repetition detection during search.
- FR-06: Insufficient material draw detection (KvK, KBvK, KNvK, KBvKB same-colored).
- FR-18: Perft depth 6 test in CI.

## 2. Design

### Three-fold Repetition (FR-05)
- Add a `history: Vec<u64>` field to `Position` in `src/board/mod.rs` to store hashes of previous positions.
- Update `Position::make_move(&mut self, mv: Move)`:
  - Before applying the move, push the current `self.hash` onto `self.history`.
  - If the move is "irreversible" (pawn move, capture, castling, or rights change), clear `self.history`.
  - Use the `halfmove_clock == 0` signal as the primary indicator for irreversible moves.
- Update `Position::make_null_move(&mut self)` to also handle the history.
- Add `Position::is_draw_by_repetition(&self) -> bool`:
  - Iterate through `self.history` and count how many times `self.hash` has already appeared.
  - Return true if the count is >= 2 (meaning this is the third occurrence).
- Add `Position::is_draw_by_repetition_with_ply(&self, ply: usize) -> bool`:
  - Search back in the history, but only as far as the current `halfmove_clock`.
- Update `alpha_beta` in `src/search/alphabeta.rs`:
  - At the start of a node (if `ply > 0`), check `pos.is_draw_by_repetition()`.
  - If true, return `0`.

### Insufficient Material (FR-06)
- Add `Position::is_draw_by_insufficient_material(&self) -> bool` to `src/board/mod.rs`.
- Logic:
  - If any pawns exist, it's not a draw.
  - If any rooks or queens exist, it's not a draw.
  - If both sides have only a King: true.
  - If one side has K and the other has KB or KN: true.
  - If both sides have KB and the bishops are on the same-colored squares: true.
- Update `alpha_beta` in `src/search/alphabeta.rs`:
  - At the start of a node (if `ply > 0`), check `pos.is_draw_by_insufficient_material()`.
  - If true, return `0`.

### Perft Depth 6 (FR-18)
- Add a test `perft_startpos_depth_6` to `tests/perft_tests.rs`.
- `assert_eq!(perft(&pos, 6), 119_060_324)`.
- Use `#[cfg_attr(debug_assertions, ignore)]` if the test is too slow for debug builds.

## 3. Files to Modify

- `src/board/mod.rs`: Update `Position` with history, irreversible-move logic, and draw-check functions.
- `src/search/alphabeta.rs`: Integrate repetition and insufficient-material checks in search.
- `tests/perft_tests.rs`: Add depth 6 test.

## 4. Verification

- `cargo test tests/perft_tests.rs -- --ignored` (if ignored).
- Add specific unit tests in `src/board/mod.rs` for `is_draw_by_repetition` and `is_draw_by_insufficient_material`.
- Verify three-fold detection with a known repetitive line in a test.
