# Task 5: Check Extension (FR-04)

Extend depth by 1 ply when in check.

## 1. Requirements

- FR-04: Depth + 1 when side to move is in check.

## 2. Design

### Check Extension
- In `src/search/alphabeta.rs`, within the move loop, compute `let next_in_check = next_pos.is_in_check(next_pos.side_to_move());`.
- Calculate `extension = if next_in_check { 1 } else { 0 };`.
- Pass `depth - 1 + extension` to the child `search` calls (both full window and null window PVS calls).
- This happens before `depth <= 0` leaf detection in the parent.

## 3. Files to Modify

- `src/search/alphabeta.rs`

## 4. Verification

- Behavioural test in `alphabeta.rs`: check extension test.
- `cargo test --all`
- Run gauntlet `just measure-elo 20 5 20`.
- Keep if `post_elo > baseline`, else revert.
