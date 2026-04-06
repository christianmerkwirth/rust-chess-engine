# Task 7: Late Move Reductions (FR-06)

Add Late Move Reductions (LMR) for quiet moves in alpha-beta.

## 1. Requirements

- FR-06: Reduce depth for late quiet moves, re-search at full depth on fail-high.

## 2. Design

### LMR Logic
- Splice into `src/search/alphabeta.rs` move loop, inside the null-window PVS branch.
- Constant `LMR_MIN_MOVE_INDEX = 4` (or similar).
- Guards: `i >= LMR_MIN_MOVE_INDEX`, `!is_pv` (null window), `!in_check` (parent), `!is_capture_or_promotion(mv)`, `!next_in_check` (child).
- Precompute an LMR reduction table via a `const fn` or `OnceLock`: `R = log(depth) * log(i) / C` (or simpler like `1` or `2` depending on depth/i, tag `// TODO: tune`).
- Run reduced search `search(..., depth - 1 - R + ext, ...)`.
- If `score > alpha`, re-search at full depth `search(..., depth - 1 + ext, ...)`.

## 3. Files to Modify

- `src/search/alphabeta.rs`

## 4. Verification

- `cargo test --all`
- Run gauntlet `just measure-elo 20 5 20`.
- Keep if `post_elo > baseline`, else revert.
