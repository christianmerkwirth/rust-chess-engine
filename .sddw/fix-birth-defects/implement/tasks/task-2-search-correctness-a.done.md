# Task Completion: Search Correctness - Part A

Implemented three-fold repetition detection, insufficient material detection, and added perft depth 6 tests.

## 1. Work Completed

### Three-fold Repetition (FR-05)
- Added `history: Vec<u64>` to `Position` in `src/board/mod.rs`.
- Updated `Position::make_move` and `Position::make_null_move` to push the current hash to history.
- `Position::make_move` clears the history if a move is irreversible (pawn move, capture, castling, or rights change).
- Implemented `Position::is_draw_by_repetition()` which checks if the current hash appears 2 or more times in the history (meaning this is the 3rd occurrence).

### Insufficient Material (FR-06)
- Implemented `Position::is_draw_by_insufficient_material()` in `src/board/mod.rs`.
- Detects draws for:
  - K vs K
  - K vs KB
  - K vs KN
  - KB vs KB (same-colored bishops)
- Added `Square::is_light()` and `Square::is_dark()` to `src/types.rs` to support bishop color detection.

### Search Integration
- Updated `alpha_beta` in `src/search/alphabeta.rs` to check for repetition and insufficient material draws at the start of each node (for `ply > 0`).

### Perft Depth 6 (FR-18)
- Added perft depth 5 and 6 tests to `tests/perft_tests.rs`.
- Depth 6 is ignored in debug mode using `#[cfg_attr(debug_assertions, ignore)]` to keep tests fast.

## 2. Verification Results

### Unit Tests
- `board::tests::test_is_draw_by_repetition`: **Passed**
- `board::tests::test_is_draw_by_insufficient_material_*`: **Passed** (multiple tests for K, KB, KN, KBvKB same/diff color)
- `board::tests::test_zobrist_incremental_*`: **Passed**
- `search::alphabeta::tests`: **Passed**

### Perft Tests
- `test_perft_startpos` (up to depth 5): **Passed** (14.18s in debug mode)
- `test_perft_kiwipete`, `test_perft_pos3..6`: **Passed**

## 3. Next Steps

Implement Search Correctness - Part B (Quiescence Search improvements and Check Extensions).

`/sddw:implement fix-birth-defects --task 3`
