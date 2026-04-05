# Task 5: Implement alpha-beta search with iterative deepening, transposition table, and move ordering

## Trace
- **FR-IDs:** FR-03, FR-05
- **Depends on:** task-2, task-3, task-4

## Files
- `src/search/mod.rs` — create
- `src/search/alphabeta.rs` — create
- `src/search/tt.rs` — create
- `src/search/ordering.rs` — create
- `src/lib.rs` — modify (add `search` module)

## Architecture

### Components
- `search` module: iterative deepening entry point, search state, stop control — new
- `alphabeta` module: negamax alpha-beta with quiescence search — new
- `tt` module: transposition table with lock-free access (AtomicU64 XOR trick) — new
- `ordering` module: move ordering (PV move, MVV-LVA, killer moves) — new

### Data Flow
```
iterative_deepening(pos, limits) →
  for depth in 1.. →
    alphabeta(pos, depth, -INF, +INF) →
      tt::probe(hash) → check for stored result
      ordering::order_moves(moves, pv_move, killers) → sorted move list
      for each move →
        child = pos.clone() + make_move
        if capture and depth == 0 → quiescence_search(child)
        else → alphabeta(child, depth-1, -beta, -alpha)
      tt::store(hash, depth, score, flag, best_move)
    report PV via info callback
  until stop signal or depth limit
→ return best move
```

## Contracts

### Internal Interfaces

**search/mod.rs:**
- `SearchLimits` struct: depth limit, node limit, stop flag (`Arc<AtomicBool>`)
- `SearchInfo` struct: current depth, nodes searched, score, PV line, time elapsed
- `SearchResult` struct: best move, score, depth reached, nodes searched
- `iterative_deepening(pos: &Position, tt: &TranspositionTable, limits: &SearchLimits, info_callback: impl FnMut(SearchInfo)) -> SearchResult`
  - Pre: position is valid, movegen initialized
  - Post: returns best move found, reports info at each depth via callback
  - Respects stop flag — checks periodically (every N nodes) and exits cleanly
- `SearchState` struct: killer moves table, history heuristic table, node counter, stop flag reference

**search/alphabeta.rs:**
- `alphabeta(pos: &Position, state: &mut SearchState, tt: &TranspositionTable, depth: i32, alpha: i32, beta: i32) -> i32`
  - Negamax formulation: score relative to side to move
  - At depth 0: call `quiescence()`
  - TT probe at entry: exact → return, lower/upper → adjust alpha/beta, cutoff if possible
  - Null move pruning (R=2): skip if in check or in endgame (optional enhancement)
  - TT store before return
- `quiescence(pos: &Position, state: &mut SearchState, tt: &TranspositionTable, alpha: i32, beta: i32) -> i32`
  - Stand-pat evaluation
  - Search only captures (via `generate_captures()`)
  - No depth limit (but limited by captures running out)
  - Delta pruning: skip captures that can't possibly raise alpha

**search/tt.rs:**
- `TTEntry` struct: two `AtomicU64` fields (XOR trick for lock-free access)
  - Logical fields packed into the two u64s: hash verification key, depth, score, flag (exact/lower/upper), best move
- `TranspositionTable` struct: `Vec<TTEntry>` with power-of-2 size
  - `TranspositionTable::new(size_mb: usize) -> TranspositionTable`
  - `TranspositionTable::probe(&self, hash: u64) -> Option<TTData>`: read entry, verify via XOR
  - `TranspositionTable::store(&self, hash: u64, data: TTData)`: write entry using XOR trick
  - `TranspositionTable::clear(&self)`: zero all entries (for `ucinewgame`)
  - `TranspositionTable::hashfull(&self) -> u32`: permille occupancy (for UCI info)
  - Uses `Relaxed` atomic ordering (XOR trick handles torn reads)
- `TTData` struct: depth (`i8`), score (`i16`), flag (`TTFlag`), best_move (`Move`)
- `TTFlag` enum: `Exact`, `LowerBound`, `UpperBound`

**search/ordering.rs:**
- `order_moves(moves: &mut MoveList, pv_move: Move, killers: &[Move; 2], pos: &Position)`
  - Priority order:
    1. PV move (from previous iteration or TT)
    2. Captures ordered by MVV-LVA (Most Valuable Victim - Least Valuable Attacker)
    3. Killer moves (2 per ply — non-capture moves that caused beta cutoff)
    4. Remaining quiet moves (ordered by history heuristic if available)
  - Pre: `moves` contains legal moves
  - Post: `moves` reordered in-place by priority
- `mvv_lva_score(attacker: Piece, victim: Piece) -> i32`: MVV-LVA score
  - `victim_value * 10 - attacker_value` (higher is better: PxQ > QxP)
- `KillerTable` struct: `[[Move; 2]; MAX_PLY]` — two killer slots per ply
  - `KillerTable::store(&mut self, ply: usize, mv: Move)`: shift and insert
  - `KillerTable::is_killer(&self, ply: usize, mv: Move) -> bool`

## Design Decisions

### Search algorithm: negamax alpha-beta
- **Chosen:** Negamax formulation of alpha-beta with fail-soft
- **Rationale:** Standard, well-understood. Negamax simplifies the code (no min/max branching). Fail-soft allows TT to store bounds more accurately.
- **Rejected:** PVS (Principal Variation Search) — would be a good enhancement later but adds complexity for v1; MTD(f) — less flexible with move ordering

### Transposition table: lock-free XOR trick
- **Chosen:** Two `AtomicU64` per entry with XOR verification (Hyatt/Mann technique)
- **Rationale:** Enables Lazy SMP without any locks. Torn reads are detected by XOR mismatch and simply ignored. `Relaxed` ordering is sufficient since correctness doesn't depend on memory ordering — only on hash verification.
- **Rejected:** Mutex-protected TT — unacceptable contention in multi-threaded search; single `AtomicU128` — not available on all platforms

### Move ordering: PV + MVV-LVA + killers
- **Chosen:** Three-tier ordering: PV move first, then captures by MVV-LVA, then killer moves, then remaining
- **Rationale:** PV move from iterative deepening is almost always best. MVV-LVA is simple and effective for capture ordering. Killer moves catch strong quiet moves. This combination achieves good pruning at ~1800 ELO level.
- **Rejected:** History heuristic alone — less effective without killers; SEE (Static Exchange Evaluation) for capture ordering — more accurate but complex, overkill for v1

### Quiescence search: captures only with stand-pat
- **Chosen:** Search all captures at depth 0 with stand-pat cutoff
- **Rationale:** Avoids horizon effect where the engine stops searching in the middle of a capture sequence. Stand-pat provides a lower bound. Simple and effective.
- **Rejected:** No quiescence — produces wildly inaccurate evaluations; quiescence with checks — adds complexity, minor benefit at this level

## Acceptance Criteria

### FR-03: Search
- GIVEN a position with a forced mate in 3
- WHEN the engine searches to sufficient depth
- THEN it SHALL find the mating move

- GIVEN iterative deepening
- WHEN a search completes depth N
- THEN the PV from depth N SHALL be used to order moves at depth N+1

- GIVEN a transposition table entry with sufficient depth
- WHEN the same position is reached again
- THEN the stored evaluation SHALL be reused to avoid redundant search

### FR-05: Move ordering
- GIVEN a position with the PV move available
- WHEN moves are ordered
- THEN the PV move SHALL be searched first

- GIVEN capturing moves
- WHEN ordered by MVV-LVA
- THEN capturing a queen with a pawn SHALL be ordered before capturing a pawn with a queen

## Done Criteria
- [ ] Iterative deepening searches depths 1, 2, 3, ... and reports info at each depth
- [ ] Alpha-beta returns correct minimax value for simple positions (verified against known evaluations)
- [ ] Engine finds mate-in-1, mate-in-2, and mate-in-3 in test positions
- [ ] TT probe correctly returns stored entries and rejects corrupted ones
- [ ] TT store/probe round-trips correctly for all TTFlag types
- [ ] `hashfull()` reports reasonable occupancy after search
- [ ] PV move is searched first when available from TT or previous iteration
- [ ] MVV-LVA orders PxQ before QxP
- [ ] Killer moves are stored and used in ordering (verify via node count reduction)
- [ ] Quiescence search avoids horizon effect (e.g., doesn't blunder hanging pieces)
- [ ] Stop flag is respected — search terminates within a few milliseconds of stop
- [ ] Node count is tracked and reported correctly
- [ ] Search handles stalemate (return 0) and checkmate (return -MATE + ply) correctly
- [ ] Unit tests for TT, move ordering, and search results on known positions
