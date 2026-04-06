# Task 9: King Safety (FR-08)

Add a king-safety term penalizing enemy attacks near the king and missing pawn shields.

## 1. Requirements

- FR-08: Count enemy attackers around the friendly king. Penalise missing/advanced pawn-shield pawns. Taper to 0 in endgame.

## 2. Design

### King Safety Logic
- Create `fn king_safety(pos: &Position, side: Color) -> (i32, i32)` in `src/eval/mod.rs`.
- Determine king square. King zone = `king_attacks(king_sq) | king_sq`.
- For each enemy piece, check if its attacks intersect the king zone.
- Sum weights for attacking pieces (e.g., N=2, B=2, R=3, Q=5) into `attack_units`.
- Map `attack_units` to a penalty using a lookup table or formula.
- Apply pawn shield penalty if pawns on king file and adjacent files are missing or advanced (especially for castled kings).
- Subtract total penalty from `mg[side]`. Return `(-penalty, 0)` to ensure it tapers to 0 in endgame.
- Add to `mg/eg` accumulators in `evaluate()`.

## 3. Files to Modify

- `src/eval/mod.rs`

## 4. Verification

- Symmetry test in `mod.rs`.
- `cargo test --all`
- Run gauntlet `just measure-elo 20 5 20`.
- Keep if `post_elo > baseline`, else revert.
