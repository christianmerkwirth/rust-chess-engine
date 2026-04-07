# Task 8 Completion: Mobility Term (REJECTED)

Added mobility term to evaluation, but it led to a significant Elo drop and was therefore reverted.

## 1. Work Performed
- Implemented mobility calculation for Knight, Bishop, Rook, and Queen.
- Added mobility tables `MOB_MG` and `MOB_EG` to `src/eval/pst.rs`.
- Updated `evaluate` in `src/eval/mod.rs` to include mobility bonus/penalty based on pseudo-legal attacks (then switched to "safe" mobility).
- Added `magics::init()` to unit tests in `src/eval/mod.rs`.

## 2. Verification Results
- Unit tests: PASS.
- Benchmark: NPS dropped by ~7.5%.
- ELO gauntlet:
  - First attempt (simple pseudo-legal): -20.9 +/- 91.4
  - Second attempt (conservative values): -63.2 +/- 95.1
  - Third attempt (safe mobility): +0.0 +/- 90.1
- Baseline: +70.4.

Since all attempts were significantly below the baseline, the changes were reverted.

## 3. Findings
- Mobility term might be double-counting "activity" already captured by the detailed PeSTO PSTs.
- Tuning mobility values is difficult without a more robust tuning process (like CLOP or SPSA).
- The NPS drop might also contribute to the Elo loss.
