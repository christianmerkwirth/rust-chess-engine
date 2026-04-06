# Task Completion Report: Task 1 - Evaluation Fixes

## 1. Work Completed

- **PST Indexing Fix (FR-01)**: Modified `src/eval/mod.rs` to correctly index PST tables. White now flips the rank (`sq ^ 56`) to match the BERF (visual) layout, while Black uses the LERF square directly, which correctly maps Black's ranks to the White-perspective tables.
- **King PST Update (FR-02)**: Replaced the King MG PST in `src/eval/pst.rs` with the values provided in the design task, which correctly penalize the king moving to the center in the middlegame.
- **Duplicated PST Rows (FR-03)**: Fixed duplicated and incorrect rows in `PST_MG[Bishop]` and `PST_MG[Queen]` in `src/eval/pst.rs`, using authentic PeSTO values.
- **Tapered Material Values (FR-04)**: Updated `MATERIAL_MG` and `MATERIAL_EG` in `src/eval/pst.rs` to use distinct PeSTO values for all pieces.
- **Eval Sanity Tests (FR-19)**: Added a comprehensive set of regression tests in `src/eval/mod.rs` that verify:
    - `e2e4` improves white's evaluation compared to the starting position.
    - A white pawn on a7 is evaluated higher than a white pawn on a2.
    - The white king prefers the back rank/castling squares over the center in the middlegame.
    - Bishop MG row 0 and row 7 are not identical.
    - MG and EG material values are distinct.

## 2. Verification Results

### Automated Tests
Ran `cargo test eval::tests` and all 11 tests passed:
- `test_compute_phase_endgame`: OK
- `test_compute_phase_startpos`: OK
- `test_evaluate_material_advantage`: OK
- `test_eval_e2e4_improves_white`: OK
- `test_eval_a7_beats_a2`: OK
- `test_evaluate_side_to_move`: OK
- `test_evaluate_startpos`: OK
- `test_evaluate_symmetry`: OK
- `test_material_mg_ne_eg`: OK
- `test_king_mg_prefers_back_rank`: OK
- `test_pst_bishop_row7_not_duplicate`: OK

## 3. Adherence to Prohibitions

- **FR-P1 (No strength work)**: No LMR, SEE, or other deferred features were implemented.
- **FR-P2 (No NNUE)**: No neural network evaluation was introduced.
