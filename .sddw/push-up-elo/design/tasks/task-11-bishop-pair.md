# Task 11: Bishop Pair (FR-10)

Add a bonus for possessing the bishop pair.

## 1. Requirements

- FR-10: Bonus when a side has at least two bishops. Tapered MG/EG.

## 2. Design

### Bishop Pair Logic
- In `src/eval/mod.rs`, after piece loops, check `pos.pieces(side, Bishop).count() >= 2`.
- Add `BISHOP_PAIR_MG` and `BISHOP_PAIR_EG` to `mg[side]` and `eg[side]`.

## 3. Files to Modify

- `src/eval/mod.rs`

## 4. Verification

- Symmetry test in `mod.rs`.
- `cargo test --all`
- Run gauntlet `just measure-elo 20 5 20`.
- Keep if `post_elo > baseline`, else revert.
