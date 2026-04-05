# Task 9: Implement Lazy SMP multi-threaded search

## Trace
- **FR-IDs:** FR-11
- **Depends on:** task-5, task-6

## Files
- `src/search/smp.rs` — create
- `src/search/tt.rs` — modify (ensure thread-safe access via Arc)
- `src/search/mod.rs` — modify (integrate SMP thread spawning)
- `src/uci.rs` — modify (wire Threads UCI option)

## Architecture

### Components
- `smp` module: Lazy SMP thread pool management — new
- `tt` module: verified thread-safe (already lock-free via XOR trick, wrapped in `Arc`) — modified
- `search` module: extended to spawn/join helper threads — modified

### Data Flow
```
UCI "go" with Threads=N →
  Main thread (thread 0):
    iterative_deepening(pos, tt, limits) — normal search
  Helper threads (1..N-1):
    spawned with same position + shared Arc<TT> + shared Arc<AtomicBool> stop flag
    each runs iterative_deepening with slightly varied parameters
    → write results to shared TT (lock-free)

On stop/time-up:
  Set stop flag → all threads check and exit
  Main thread waits for helpers to join
  Best move from main thread (thread 0) is reported

Thread diversification:
  Helper threads vary search depth order to explore different parts of the tree
  e.g., thread i starts from depth (1 + i%2) and may skip some depths
```

## Contracts

### Internal Interfaces

**search/smp.rs:**
- `ThreadPool` struct: manages helper search threads
  - `ThreadPool::new(num_threads: usize) -> ThreadPool`
  - `ThreadPool::search(&self, pos: &Position, tt: Arc<TranspositionTable>, limits: &SearchLimits, info_callback: impl FnMut(SearchInfo)) -> SearchResult`
    - Spawns `num_threads - 1` helper threads
    - Main thread (thread 0) runs normal iterative deepening and handles `info` output
    - Helper threads run iterative deepening with depth variation, writing to shared TT
    - All threads share: `Arc<TranspositionTable>`, `Arc<AtomicBool>` stop flag
    - On completion: set stop flag, join all helpers, return main thread's result
  - `ThreadPool::stop(&self)`: set stop flag (called from UCI on `stop` command)
  - `ThreadPool::resize(&mut self, num_threads: usize)`: change thread count (for UCI option)

**Thread diversification strategy:**
- Thread 0 (main): standard iterative deepening 1, 2, 3, 4, ...
- Thread i (helper): start from depth `1 + (i % 3)`, or vary aspiration windows
  - The "lazy" in Lazy SMP: no work splitting, no communication between threads — they just search independently and share the TT
  - Diversity comes naturally from different search order and TT interactions

**search/tt.rs (verification):**
- `TranspositionTable` already uses `AtomicU64` XOR trick (task 5)
- Verify: `TranspositionTable` is `Send + Sync` (required for `Arc` sharing)
- Verify: no `&mut self` methods are called during search (all access via shared reference)

**search/mod.rs (changes):**
- `iterative_deepening()` takes `thread_id: usize` parameter
  - Thread 0: reports info, tracks best move
  - Thread 1+: searches silently, contributes to TT only
- Each thread gets its own `SearchState` (killers, node count) — not shared

**uci.rs (changes):**
- `Threads` UCI option wired to `ThreadPool::resize()`
- Default: 1 thread (single-threaded mode is just ThreadPool with N=1)

## Design Decisions

### Multi-threading approach: Lazy SMP
- **Chosen:** Lazy SMP — all threads search the same position independently, sharing only the transposition table
- **Rationale:** Simple to implement, well-proven (used by Stockfish, Ethereal, and many others). No complex work splitting or lock management. Threads naturally explore different parts of the tree due to TT interactions and slight depth variation. Scales well up to 8-16 threads.
- **Rejected:** YBWC (Young Brothers Wait Concept) — complex work splitting, hard to implement correctly; SHT (Shared Hash Table only, no diversification) — works but less effective without depth variation

### Thread communication: shared TT + atomic stop flag only
- **Chosen:** No inter-thread communication except shared TT and stop flag. Each thread has its own search state (killers, history, node counter).
- **Rationale:** Eliminates all synchronization overhead. The TT naturally propagates good results between threads. Individual search state avoids contention on killer/history tables.
- **Rejected:** Shared killer table — contention outweighs benefits; message passing — overcomplicates a "lazy" approach

### Best move selection: main thread (thread 0)
- **Chosen:** Always report the best move from thread 0 (the main thread)
- **Rationale:** Thread 0 runs standard iterative deepening with proper PV tracking. Helper threads contribute to the TT which benefits thread 0's search, but their individual results may be from non-standard depth ordering. This is the standard Lazy SMP approach.
- **Rejected:** Best move from any thread — requires comparing results across different search depths, adds complexity

## Acceptance Criteria

### FR-11: Multi-threaded search
- GIVEN the `Threads` UCI option is set to N
- WHEN the engine searches
- THEN it SHALL use N threads with a shared transposition table (Lazy SMP)

- GIVEN multi-threaded search
- WHEN all threads complete
- THEN the best move from the main thread SHALL be reported

## Done Criteria
- [ ] `ThreadPool` correctly spawns N-1 helper threads alongside the main thread
- [ ] All threads share the same `TranspositionTable` via `Arc`
- [ ] All threads respect the shared stop flag
- [ ] Helper threads exit cleanly when stop flag is set
- [ ] `ThreadPool::search()` joins all threads before returning
- [ ] Main thread (thread 0) reports info and returns the best move
- [ ] Search with Threads=1 is functionally identical to single-threaded search
- [ ] Search with Threads=4 produces a valid best move (correctness maintained)
- [ ] Multi-threaded search shows higher NPS than single-threaded (performance test)
- [ ] No data races or undefined behavior (verified with `cargo test` under Miri or ThreadSanitizer)
- [ ] `TranspositionTable` is `Send + Sync`
- [ ] UCI `setoption name Threads value N` correctly resizes the thread pool
- [ ] Thread 0 performs standard iterative deepening; helpers use depth variation
- [ ] Integration test: engine responds correctly to go/stop with multiple threads
