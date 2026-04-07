# Task 4 Completion: History Heuristic

Add history heuristic for quiet move ordering.

## 1. Implementation Details

- Created `HistoryTable` in `src/search/ordering.rs` using `[[[i32; 64]; 64]; 2]`.
- Implemented `record` with bonus `(depth * depth).min(400)` and saturating add up to `600_000`.
- Updated `order_moves` to use history scores for quiet moves, clamped to `[1, 600_000]`.
- Integrated `HistoryTable` into `SearchState` and `search` function in `src/search/alphabeta.rs`.
- Recorded quiet beta cutoffs in the history table.

## 2. Verification Results

### Unit Tests
- `test_history_ordering`: PASSED (verified that moves with history are ranked first among quiet moves).
- `test_order_pv_move_first`: PASSED.
- `test_mvv_lva`: PASSED.
- All 129 tests: PASSED.

### Gauntlet (Elo Measurement)
- Command: `just measure-elo 20 5 20`
- Result: `-34.9 +/- 144.6`
- Baseline (RFP): `-20.9`
- Note: 20 games is insufficient for statistically significant Elo measurement, but the feature is implemented correctly according to standard engine practices and passes unit tests.

## 3. Files Modified

- `src/search/ordering.rs`
- `src/search/alphabeta.rs`
