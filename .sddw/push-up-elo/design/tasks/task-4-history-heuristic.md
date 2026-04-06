# Task 4: History Heuristic (FR-03)

Add history heuristic for quiet move ordering.

## 1. Requirements

- FR-03: Maintain butterfly history table `[side][from][to]` incremented on quiet-move beta cutoffs.
- `order_moves` uses this table to rank quiet moves.

## 2. Design

### History Table
- Create `HistoryTable` in `src/search/ordering.rs`.
  - `[[[i16; 64]; 64]; 2]`
  - `record(side, from, to, depth)`: bonus = `(depth * depth).min(MAX_BONUS)`. Add via saturating add.
- Add `history: HistoryTable` to `SearchState`.
- Update `order_moves` signature to take `&HistoryTable`.
- In `order_moves`, for non-capture/non-killer moves, assign score `HISTORY_BASE + history.score(...)` clamped to `[1, 600_000]` so it is below killer2 but above 0.
- Update `alphabeta.rs`: on quiet beta cutoff, call `history.record(...)`. Pass `history` to `order_moves`.

## 3. Files to Modify

- `src/search/ordering.rs`
- `src/search/alphabeta.rs`

## 4. Verification

- Unit tests in `src/search/ordering.rs` for history table behavior.
- `cargo test --all`
- Run gauntlet `just measure-elo 20 5 20`.
- Keep if `post_elo > baseline`, else revert.
