# Task 10 Completion: Pawn Structure (REJECTED)

Added pawn-structure terms for passed, isolated, and doubled pawns, but it resulted in a regression and was therefore reverted.

## 1. Work Performed
- Implemented `evaluate_pawns` function in `src/eval/mod.rs`.
- Added `FILE_MASKS` and `ADJACENT_FILE_MASKS` bitboard constants.
- Added `passed_pawn_mask`, `ranks_above`, and `ranks_below` helper functions for passed pawn detection.
- Defined `PASSED_MG/EG`, `ISOLATED_MG/EG`, and `DOUBLED_MG/EG` constants in `src/eval/pst.rs`.
- Integrated `evaluate_pawns` into the tapered evaluation in `evaluate()`.
- Added unit tests for doubled, isolated, and passed pawns.

## 2. Verification Results
- Unit tests: PASS.
- ELO gauntlet (vs Stockfish Skill 5, 10+0.1):
  - 20 games: +0.0 +/- 153.5
  - 50 games: -70.4 +/- 91.9
- Baseline (from Task 7): +70.4.

Since the results showed a significant regression compared to the baseline, the changes were reverted to preserve the engine's performance.

## 3. Findings
- The PeSTO Piece-Square Tables already include significant bonuses for pawn advancement and center control, which may partially account for pawn structure values. Adding additional explicit terms without an automated tuning framework (like Texel tuning) likely leads to double-counting and imbalances in the evaluation function.
- Doubled and isolated pawn penalties might have been too aggressive given the existing PST-based evaluation.
- Like with mobility (Task 8) and king safety (Task 9), adding complex evaluation terms is proving difficult without a way to precisely tune the weights.
