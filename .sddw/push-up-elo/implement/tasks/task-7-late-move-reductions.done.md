# Task 7 Completion Report: Late Move Reductions (LMR)

## Implementation Details

- Added a precomputed LMR reduction table in `src/search/alphabeta.rs` using `OnceLock`.
- The reduction table is calculated using the standard formula: `R = 0.75 + ln(depth) * ln(move_index) / 2.25`.
- Integrated LMR logic into the move loop of the `search` function for non-PV moves.
- Added guards for LMR:
    - `depth >= 3`
    - `i >= 4` (move index, starting from the 5th move)
    - Not in check (parent node)
    - Move is not a capture or promotion
    - Move does not give check (extension is 0)
- Implemented the re-search mechanism:
    - If the reduced search fails high, re-search at full depth with a null window.
    - If it still fails high, re-search with a full window.

## Files Modified

- `src/search/alphabeta.rs`

## Verification Results

### Unit Tests
- `cargo test --all` passed (131 tests).

### Elo Measurement
- Ran `just measure-elo 20 5 20`.
- Baseline Elo: `-17.4`
- Post-implementation Elo: `+70.4 +/- 158.3`
- Delta: `+87.8`
- Verdict: SUCCESS
