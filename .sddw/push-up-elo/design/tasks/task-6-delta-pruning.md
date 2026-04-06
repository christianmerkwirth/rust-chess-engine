# Task 6: Delta Pruning (FR-05)

Add delta pruning to quiescence search.

## 1. Requirements

- FR-05: Skip captures in quiescence if `stand_pat + victim_value + DELTA_MARGIN < alpha`.
- Do not prune promotions.

## 2. Design

### Delta Pruning Logic
- In `src/search/alphabeta.rs` inside `quiescence` move loop.
- Execute before `next_pos.clone()`.
- Constant: `DELTA_MARGIN = 200` (or similar).
- Victim value: `PIECE_VALUES[pos.piece_at(mv.to_sq()).map(|(_, p)| p as usize).unwrap_or(0)]`. For en-passant, `PIECE_VALUES[Pawn]`.
- Condition: `!mv.is_promotion() && stand_pat + victim_value + DELTA_MARGIN < alpha`. If true, `continue`.
- Make `PIECE_VALUES` public in `ordering.rs`.

## 3. Files to Modify

- `src/search/alphabeta.rs`
- `src/search/ordering.rs` (pub const)

## 4. Verification

- Unit test in `alphabeta.rs` for delta pruning predicate.
- `cargo test --all`
- Run gauntlet `just measure-elo 20 5 20`.
- Keep if `post_elo > baseline`, else revert.
