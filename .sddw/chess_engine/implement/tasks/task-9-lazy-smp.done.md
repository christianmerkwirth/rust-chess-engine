# Task 9 Completion: Implement Lazy SMP multi-threaded search

## Summary
Created `src/search/smp.rs` with a `ThreadPool` struct that spawns N-1 helper threads each running independent iterative deepening with slight depth variation (1 + i%3), sharing a single `Arc<TranspositionTable>` and `Arc<AtomicBool>` stop flag. The main thread (thread 0) runs standard search and reports info; helpers contribute silently to the shared TT. Wired `Threads` UCI option to `ThreadPool::resize`.

## Commits
- `6b4231f` test(chess-engine): add failing tests for Lazy SMP (FR-11)
- `902f373` feat(chess-engine): implement Lazy SMP multi-threaded search (FR-11)

## Deviations
- **Rule 1: Bug** — `src/search/tt.rs` and `tests/uci_tests.rs` were created in a prior task session but never committed (shown as `??` in git status). They were included in the RED commit, not a separate one. No functional impact.

## Difficulties
- None — `TranspositionTable` was already lock-free via the XOR trick with `AtomicU64`, making it naturally `Send + Sync` with no modifications required.

## Notes
- `ThreadPool::num_threads` is pub so `uci.rs` can read it when cloning into the spawned search thread. Consider making it a method if the API is ever stabilised.
- `uci.rs` now always routes through `ThreadPool::search`, even for 1 thread. With 1 thread, no helpers are spawned and behaviour is identical to the previous direct `iterative_deepening` call.
- The stop flag is shared: time-limit fires in any thread set it for all threads, so helpers naturally stop when the main thread's time expires.
