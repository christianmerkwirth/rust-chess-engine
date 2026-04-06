# Task 3: Reverse Futility Pruning (FR-02)

Add Reverse Futility Pruning (RFP) to the alpha-beta search.

## 1. Requirements

- FR-02: Non-PV, non-in-check search applies RFP at shallow depth.
- Guard: `depth <= RFP_MAX_DEPTH`, `!in_check`, `ply > 0`, `!is_pv`, `abs(beta) < MATE_SCORE - 1000`.
- Body: `if static_eval - margin >= beta { return static_eval }`.

## 2. Design

### RFP Logic
- Splice into `src/search/alphabeta.rs` between tablebase probe and NMP.
- Constants: `RFP_MAX_DEPTH = 6` (or similar, tag with `// TODO: tune`), `margin(d) = 80 * d` (or similar).
- Need `!is_pv`, which is approximated by `beta - alpha <= 1` (null window) if PV flag isn't passed explicitly. 
- If conditions met, compute `static_eval = evaluate(pos)`. (Use existing `evaluate`).
- Prune if `static_eval - margin(depth) >= beta`, returning `static_eval`.

## 3. Files to Modify

- `src/search/alphabeta.rs`

## 4. Verification

- `cargo test --all`
- Run gauntlet `just measure-elo 20 5 20`.
- Keep if `post_elo > baseline`, else revert.
