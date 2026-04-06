# Task 8: Mobility Term (FR-07)

Add a mobility term to the static evaluation.

## 1. Requirements

- FR-07: Evaluate pseudo-legal attack squares for N, B, R, Q (excluding friendly occupied squares).
- Tapered between MG and EG.

## 2. Design

### Mobility Logic
- In `src/eval/mod.rs` inside the `piece` loop.
- If piece is N, B, R, or Q:
  - Calculate `attacks = magics::piece_attacks(...) & !us_occ`.
  - Count bits: `mob = attacks.count() as usize`.
  - Add `MOB_MG[piece][mob]` to `mg[c]` and `MOB_EG[piece][mob]` to `eg[c]`.
- Define `MOB_MG` and `MOB_EG` tables in `src/eval/pst.rs` (or `mod.rs`) for N, B, R, Q up to their max mobility (e.g., N=8, B=13, R=14, Q=27). Tag with `// TODO: tune` or cite values.

## 3. Files to Modify

- `src/eval/mod.rs`
- `src/eval/pst.rs` (if defining constants there)

## 4. Verification

- Symmetry test in `mod.rs`.
- `cargo test --all`
- Run gauntlet `just measure-elo 20 5 20`.
- Keep if `post_elo > baseline`, else revert.
