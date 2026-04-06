# Task 4: Time Management and Threading

Fix time-management bugs related to pondering and ensure proper search-thread management when new commands are received.

## 1. Requirements

- FR-10: Ponder clock preservation (reset `start_time` on `ponderhit`).
- FR-11: Stop and join in-flight search before spawning a new one.

## 2. Design

### Ponder Clock Preservation (FR-10)
- Update `SearchState` in `src/search/alphabeta.rs` to include `was_pondering: bool`.
- Initialize `was_pondering` from `pondering.load(Ordering::Relaxed)` in `SearchState::new`.
- Update the poll block in `alpha_beta` (the section that runs every 2048 nodes):
  - Check the current `pondering` flag.
  - If `state.was_pondering` is true AND the current `pondering` flag is false:
    - This is the `ponderhit` event.
    - Reset `state.start_time = Instant::now()`.
    - Set `state.was_pondering = false`.
- Ensure this change is reflected in `quiescence` once the stop-check is added there.

### Search Thread Join on New `go` (FR-11)
- Update the `"go"` branch in `src/uci.rs` (at the start of `parse_go` or before spawning the thread):
  - If `engine.search_handle.is_some()`:
    - Set `engine.stop.store(true, Ordering::Relaxed)`.
    - Clear `engine.pondering.store(false, Ordering::Relaxed)`.
    - Take the handle and call `.join().unwrap()`.
  - Reset `engine.stop.store(false, Ordering::Relaxed)` before spawning the new thread.
- Ensure any other command that starts a search (like `ponderhit` implicitly if it re-enters search logic) follows the same pattern.
- This prevents multiple search threads from competing for the TT and other shared resources.

## 3. Files to Modify

- `src/search/alphabeta.rs`: Update `SearchState` and poll logic for `ponderhit`.
- `src/uci.rs`: Update `parse_go` and thread management.

## 4. Verification

- Test by sending `go ponder`, waiting some time, then `ponderhit`, and verifying that the engine doesn't stop immediately due to burned time.
- Verify thread safety by hammering the engine with `go` commands in quick succession (e.g., using a script).
- Check `cargo test tests/uci_tests.rs` for existing UCI-level tests that can be extended.
