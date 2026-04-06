# Task 5: UCI Protocol and Search Limits

Correct UCI protocol issues, ensure `position` command atomicity, and enforce all `go` search limits correctly.

## 1. Requirements

- FR-12: Correct `id author Christian Merkwirth`.
- FR-13: `position` command atomicity (no partial application on failure).
- FR-14: Emit `bestmove 0000` in terminal positions.
- FR-15: Verify book move legality before emitting.
- FR-16: Enforce `go nodes` and `go depth` limits.
- FR-17: Quiescence stop-check and ply bound.

## 2. Design

### UCI Author Identity (FR-12)
- Change `id author` string to `Christian Merkwirth` in `src/uci.rs`.

### `position` Atomicity (FR-13)
- Refactor `parse_position` in `src/uci.rs` to stage changes into a temporary `Position`.
- Apply all `moves` tokens to the temporary `Position`.
- If any move is unparseable or illegal, abort and keep the original `engine.pos`.
- Use `info string` to log the failure.

### `bestmove 0000` for Terminal Positions (FR-14)
- In `parse_go` (`src/uci.rs`), before starting the search:
  - Generate all legal moves for the current position.
  - If no legal moves exist (checkmate or stalemate):
    - `println!("bestmove 0000")`.
    - Return without spawning the search thread.

### Book Move Legality (FR-15)
- In `parse_go` (`src/uci.rs`), if a book move is found:
  - Generate legal moves for the current position.
  - Verify that the book move exists in the legal move list.
  - If not legal, log an `info string` and fall back to the normal search loop.

### `go nodes` and `go depth` Enforcement (FR-16)
- Update `GoParams` and `SearchLimits` to carry the `nodes` limit.
- Update `SearchState` to include `nodes_limit: Option<u64>`.
- In the alpha-beta poll block (every 2048 nodes), check if `state.nodes > state.nodes_limit` and set `state.stop` if so.
- In `iterative_deepening`, check the `nodes` and `depth` limits at the start of each iteration.

### Quiescence Safety (FR-17)
- Add `qply: usize` and `MAX_QPLY` (constant, e.g., 64) to the `quiescence` function.
- Increment `qply` in recursive calls.
- If `qply >= MAX_QPLY`, return the stand-pat score immediately.
- Integrate the stop-flag check into the `quiescence` search using the same `nodes & 0x7FF == 0` cadence as `alpha_beta`.

## 3. Files to Modify

- `src/uci.rs`: Update identity, `position` atomicity, `bestmove 0000`, and book legality.
- `src/search/alphabeta.rs`: Update `SearchState`, poll logic, and `quiescence` with stop-check and ply bound.
- `src/search/mod.rs`: Update `iterative_deepening` with `nodes`/`depth` limit checks.
- `src/time.rs`: Pass `nodes` from `GoParams` into `SearchLimits`.

## 4. Verification

- Verify `go nodes 1000` stops near the requested node count.
- Verify `go depth 5` doesn't exceed depth 5 in iterative deepening.
- Test `position startpos moves <legal> <illegal>` doesn't change the engine's position.
- Confirm `bestmove 0000` is emitted for checkmated and stalemated positions.
- Review `cargo test tests/uci_tests.rs` for regression tests on these UCI items.
