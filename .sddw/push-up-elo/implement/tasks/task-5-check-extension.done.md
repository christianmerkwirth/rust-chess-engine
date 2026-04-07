# Task 5 Completion: Check Extension

Extend search depth by 1 ply when a move gives check.

## 1. Implementation Details

- Modified `src/search/alphabeta.rs` in the main search move loop.
- After making a move, computed if the new position is in check for the side to move (`next_pos.is_in_check(next_pos.side_to_move())`).
- If so, set `extension = 1`, else `0`.
- Passed `depth - 1 + extension` to all recursive `search` calls (PV, null-window, and full-window re-search).
- This ensures that moves giving check are searched 1 ply deeper, allowing the engine to see forced sequences more clearly.

## 2. Verification Results

### Unit Tests
- `test_check_extension`: PASSED. Verified that a mate-in-2 (which usually requires depth 4) is found at depth 2 with check extensions.
- `test_mate_in_1_depth_1_with_extension`: PASSED. Verified that mate-in-1 is found at depth 1 when the move gives check.
- All 130 tests: PASSED.

### Gauntlet (Elo Measurement)
- Command: `just measure-elo 100 5 20`
- Result: `-49.0 +/- 65.9`
- Baseline (History Heuristic): `-34.9`
- Note: Although the point estimate shows a slight decrease, it is within the margin of error for 100 games. Check extension is a standard and robust engine technique that is essential for tactical strength.

## 3. Files Modified

- `src/search/alphabeta.rs`
