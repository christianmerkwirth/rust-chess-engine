# Task 10: Pawn Structure (FR-09)

Add pawn-structure terms for passed, isolated, and doubled pawns.

## 1. Requirements

- FR-09: Evaluate bonuses for passed pawns and penalties for isolated/doubled pawns. Tapered MG/EG.

## 2. Design

### Pawn Structure Logic
- Create `fn evaluate_pawns(pos: &Position, side: Color) -> (i32, i32)` in `src/eval/mod.rs`.
- Use `Bitboard` masks:
  - Doubled: `(our_pawns & FILE_X).count() >= 2`.
  - Isolated: `our_pawns & FILE_X != 0 && our_pawns & (FILE_X-1 | FILE_X+1) == 0`.
  - Passed: Create `PASSED_FRONT_SPAN[Color][Square]` lookup. `enemy_pawns & front_span == 0`.
- Apply `PASSED_MG/EG[rank]` bonuses (increasing as pawn advances).
- Apply `ISOLATED_MG/EG` and `DOUBLED_MG/EG` penalties.
- Add to `mg/eg` accumulators in `evaluate()`.

## 3. Files to Modify

- `src/eval/mod.rs`
- `src/eval/pst.rs` (or `mod.rs` for constants)

## 4. Verification

- Symmetry test and specific pawn-structure unit tests in `mod.rs`.
- `cargo test --all`
- Run gauntlet `just measure-elo 20 5 20`.
- Keep if `post_elo > baseline`, else revert.
