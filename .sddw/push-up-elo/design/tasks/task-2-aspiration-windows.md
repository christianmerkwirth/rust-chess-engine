# Task 2: Aspiration Windows (FR-01)

Add aspiration windows to the iterative deepening loop.

## 1. Requirements

- FR-01: Begin each ID iteration with an aspiration window centred on the previous iteration's score.
- Widen window progressively on fail-high/fail-low.
- Skip aspiration if `|prev_score| > MATE_SCORE - 1000`.

## 2. Design

### Aspiration Loop
- Modify `src/search/mod.rs:iterative_deepening`.
- Add `let mut prev_score: Option<i32> = None;` before the loop.
- Inside loop, replace `search(..., -INFINITY, INFINITY)` with an aspiration loop if `prev_score` is present and not a mate score.
- Initial window: `[prev_score - 50, prev_score + 50]`.
- On fail-low (`score <= alpha`): widen `alpha = alpha - delta`, `delta *= 2`.
- On fail-high (`score >= beta`): widen `beta = beta + delta`, `delta *= 2`.
- Re-search until score is within `[alpha, beta]`.
- Check `state.stop.load(...)` after each re-search to avoid infinite loop.

## 3. Files to Modify

- `src/search/mod.rs`

## 4. Verification

- `cargo test --all`
- Run gauntlet `just measure-elo 20 5 20`.
- Keep if `post_elo > baseline`, else revert.
