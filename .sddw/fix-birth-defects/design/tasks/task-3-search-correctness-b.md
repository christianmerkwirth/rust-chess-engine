# Task 3: Search Correctness - Part B

Improve Transposition Table (TT) accuracy and reliability by adjusting mate scores by ply, preventing root cutoffs, and implementing a depth-preferred replacement policy with aging.

## 1. Requirements

- FR-07: Adjust mate scores by `ply` on TT store and probe.
- FR-08: No TT cutoff at the root (`ply == 0`).
- FR-09: Depth-preferred TT replacement with generation/age counter.

## 2. Design

### TT Mate-Score Adjustment (FR-07)
- Define helper functions in `src/search/alphabeta.rs` (or `src/search/mod.rs`):
  ```rust
  fn score_to_tt(s: i32, ply: i32) -> i16 {
      if s >=  MATE_SCORE - 1000 { (s + ply) as i16 }
      else if s <= -MATE_SCORE + 1000 { (s - ply) as i16 }
      else { s as i16 }
  }
  fn score_from_tt(s: i32, ply: i32) -> i32 {
      if s >=  MATE_SCORE - 1000 { s - ply }
      else if s <= -MATE_SCORE + 1000 { s + ply }
      else { s }
  }
  ```
- Use `score_from_tt` when probing the TT in `alpha_beta`.
- Use `score_to_tt` when storing results in the TT.

### Root TT Cutoff Guard (FR-08)
- Update `alpha_beta` in `src/search/alphabeta.rs`:
  - When probing the TT, check `if ply > 0` before allowing a hard cutoff (Exact, UpperBound, or LowerBound).
  - Even at `ply == 0`, still use the TT move for ordering, but do not return early.

### TT Replacement and Aging (FR-09)
- Modify `TTEntry` and `TTData` in `src/search/tt.rs` to include a `generation: u8` field.
- Reuse the 26 free bits in the XOR-packed `word0` of `TTEntry`.
- Add an `AtomicU8` generation counter to the `TranspositionTable` struct.
- Add `TranspositionTable::new_search(&self)` which increments this counter (called on `ucinewgame`).
- Update `TranspositionTable::store`:
  - Fetch the old entry.
  - Replacement logic: `if old.generation != current || new.depth >= old.depth { replace }`.
  - Otherwise, do not replace.
- Ensure `depth` is stored as an `i8` consistently.

## 3. Files to Modify

- `src/search/alphabeta.rs`: Update TT probe/store logic and add mate-score helpers.
- `src/search/tt.rs`: Update TT data layout, replacement policy, and add generation counter.
- `src/uci.rs`: Call `tt.new_search()` on `ucinewgame`.

## 4. Verification

- Add unit tests for `score_to_tt` and `score_from_tt`.
- Test that mate scores reported by the search are consistent regardless of depth and TT hits.
- Verify TT replacement logic with a unit test that simulates storing entries at different depths and generations.
- Use `info string` to log TT hit/store stats during a `go` command (optional, for manual verification).
