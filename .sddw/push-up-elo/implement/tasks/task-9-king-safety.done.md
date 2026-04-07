# Task 9 Completion: King Safety (REJECTED)

Added a king safety term including enemy attackers near the king and a pawn shield penalty, but it resulted in a regression and was therefore reverted.

## 1. Work Performed
- Implemented `king_safety` function in `src/eval/mod.rs`.
- Calculated `attack_units` based on enemy pieces attacking the king zone (king square + adjacent squares).
- Mapped `attack_units` to a penalty using a standard `SAFETY_TABLE`.
- Implemented a pawn shield penalty for missing or advanced pawns on the files adjacent to the king.
- Integrated `king_safety` into the middlegame portion of the tapered evaluation.
- Added unit tests for symmetry, attacker counting, and pawn shield.

## 2. Verification Results
- Unit tests: PASS.
- ELO gauntlet (vs Stockfish Skill 5, 10+0.1):
  - 20 games: -52.5 +/- 151.0
  - 50 games: -20.9 +/- 95.8
- Baseline (from Task 7): +70.4.

Since the results were significantly below the baseline, the changes were reverted to maintain the engine's strength.

## 3. Findings
- The "king safety" logic might be overlapping with the existing PeSTO PSTs, which already encourage the king to stay in safe corners during the middlegame.
- The weights in the `SAFETY_TABLE` or the `ATTACK_WEIGHTS` might need finer tuning to be effective with this specific engine's evaluation.
- The pawn shield penalty might be too simplistic or aggressive.
- Similar to the mobility term (Task 8), adding complex evaluation terms without an automated tuning framework often leads to regressions.
