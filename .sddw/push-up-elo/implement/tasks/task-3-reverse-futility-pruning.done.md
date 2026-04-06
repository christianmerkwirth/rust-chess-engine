# Task Completion: Reverse Futility Pruning (FR-02)

Implemented Reverse Futility Pruning (RFP), also known as Static Null Move Pruning, in the alpha-beta search.

## Changes

### `src/search/alphabeta.rs`
- Added RFP logic before Null Move Pruning.
- Applied RFP when:
    - `depth <= 6`
    - Not in check
    - Not at the root (`ply > 0`)
    - Null window search (`beta - alpha <= 1`)
    - Not in a mate-heavy position (`beta.abs() < MATE_SCORE - 1000`)
- Pruning threshold: `static_eval - 80 * depth >= beta`.

## Verification Results

### Automated Tests
- Ran `cargo test --all`: All 128 tests passed.
- Perft tests passed.
- UCI limits tests passed.

### Elo Measurement
- Baseline: `-41.9 +/- 93.0` (50 games vs Stockfish Skill 5)
- Post-RFP: `-20.9 +/- 91.4` (50 games vs Stockfish Skill 5)
- Net Gain: `+21.0` Elo (Estimated)
