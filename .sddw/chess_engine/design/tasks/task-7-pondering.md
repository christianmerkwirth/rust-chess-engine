# Task 7: Implement pondering

## Trace
- **FR-IDs:** FR-08
- **Depends on:** task-5, task-6

## Files
- `src/uci.rs` — modify (add ponder handling)
- `src/search/mod.rs` — modify (add ponder mode support)

## Architecture

### Components
- `uci` module: extended to handle `go ponder`, `ponderhit` commands — modified
- `search` module: extended to support ponder mode (search without time limit until ponderhit/stop) — modified

### Data Flow
```
Engine sends: bestmove e2e4 ponder e7e5
GUI sends: position startpos moves e2e4 e7e5
GUI sends: go ponder
  → Engine starts searching the predicted position with no time limit
  → AtomicBool "pondering" flag set to true

Case 1 — Opponent plays predicted move:
  GUI sends: ponderhit
  → Set pondering = false
  → Apply time control (allocate remaining time)
  → Search continues with time limit until completion
  → Engine sends: bestmove ...

Case 2 — Opponent plays different move:
  GUI sends: stop
  → Abort search, discard result
  GUI sends: position ... (new position)
  GUI sends: go wtime ... (normal search)
  → Normal search on the new position
```

## Contracts

### Internal Interfaces

**uci.rs (additions):**
- `UciCommand::PonderHit` variant: added to the command enum
- `parse_command()`: handle `ponderhit` as a new command
- In the UCI loop: when `ponderhit` is received, signal the search to switch from ponder to normal mode
- `send_bestmove()`: include `ponder <move>` when a predicted reply is available (PV line has at least 2 moves)

**search/mod.rs (additions):**
- `SearchLimits` extended with `ponder: bool` field
- When `ponder == true`: ignore time limits, search indefinitely until `ponderhit` or `stop`
- When `ponderhit` received: switch to time-managed search (apply time allocation from the original `go` params that will be sent, or use reasonable defaults)
- `SearchState` extended with `pondering: Arc<AtomicBool>` — checked alongside stop flag
- Time check logic: `if pondering { continue } else { check time }`

## Design Decisions

### Ponder implementation: shared atomic flag
- **Chosen:** `AtomicBool` pondering flag, checked in the same time-check loop as the stop flag. On `ponderhit`, clear the flag and set the time allocation.
- **Rationale:** Minimal change to existing search. The search already checks a stop flag periodically; adding a ponder flag to the same check is trivial. No need to restart the search — it continues seamlessly.
- **Rejected:** Separate ponder thread — unnecessary complexity; restarting search on ponderhit — wastes all work done during pondering

### Ponder move selection: second move in PV
- **Chosen:** The ponder move is the second move in the principal variation from the last completed search iteration
- **Rationale:** The PV already predicts the opponent's best reply. Using it as the ponder move is free and usually accurate.
- **Rejected:** Book-based ponder move — only works in opening; separate ponder analysis — too complex for v1

## Acceptance Criteria

### FR-08: Pondering
- GIVEN `bestmove e2e4 ponder e7e5` was sent
- WHEN the GUI sends `go ponder`
- THEN the engine SHALL search the predicted position until `ponderhit` or `stop`

- GIVEN the engine is pondering
- WHEN `ponderhit` is received
- THEN the engine SHALL switch to normal time-managed search

## Done Criteria
- [ ] `go ponder` starts a search with no time limit
- [ ] `ponderhit` switches search to time-managed mode without restarting
- [ ] `stop` during pondering halts search and returns bestmove
- [ ] `bestmove` includes `ponder <move>` when PV has >= 2 moves
- [ ] Engine does not flag on time when pondering (time is not counted)
- [ ] Pondering → ponderhit → bestmove sequence works correctly end-to-end
- [ ] Pondering → stop → new position → go works correctly (ponder miss)
- [ ] Integration test covers both ponderhit and ponder-miss scenarios
