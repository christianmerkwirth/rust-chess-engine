# Task 1: Process Setup and Gauntlet Script Fix

Fix the latent bug in `scripts/measure_elo.py` and prepare the process infrastructure for gating patches based on Elo measurements.

## 1. Requirements

- Fix `measure_elo.py` latent bug so it correctly parses `Elo difference:` lines.
- Prepare the run log format at `.sddw/push-up-elo/run-log.md` (FR-12, FR-13, FR-14 process).

## 2. Design

### Script Fix
- Move `import math` to the top of `scripts/measure_elo.py` so that `math.isnan(e)` works inside `run_match()`.

### Run Log Infrastructure
- Create `.sddw/push-up-elo/run-log.md` with headers: `| # | patch | baseline_elo | post_elo | delta | margin | verdict | commit |`.
- Record baseline Elo at HEAD before starting the patches.

## 3. Files to Modify

- `scripts/measure_elo.py`
- `.sddw/push-up-elo/run-log.md` (Create)

## 4. Verification

- Run `uv run scripts/measure_elo.py --games 2 --sf-skill 5 --concurrency 2` and ensure it completes without raising `NameError`.
