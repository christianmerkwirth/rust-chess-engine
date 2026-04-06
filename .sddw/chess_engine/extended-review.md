# Extended Critical Review — `chess_engine`

**Verdict**: the existing `.sddw/chess_engine/verify/report.md` ("All FRs PASS") and `.sddw/chess_engine/review/review.md` ("excellent quality") are misleading. The engine has **catastrophic evaluation bugs**, **missing correctness features required by the FRs**, and a measured tournament score of **~36% against Stockfish skill 3 (7+10 wins vs 17+14 losses)** — well under the stated ~1800 ELO goal.

Move generation is correct (perft verified to depth 6 — `startpos=119,060,324`, Kiwipete depth 5 = `193,690,690`), but everything else has problems.

---

## 1. Critical bugs (strength-breaking)

### 1.1 PSTs are indexed BACKWARDS for white — the engine hates advancing pawns

`src/eval/mod.rs:27-32`:
```rust
let eval_sq = if color == Color::White {
    sq.0 as usize                  // LERF, no flip
} else {
    (sq.0 ^ 56) as usize
};
```

The PST tables in `src/eval/pst.rs` are in **PeSTO visual / BERF layout** (row 0 = rank 8). To use them from LERF squares (`a1=0`), **white must flip and black must not**. This engine does the opposite. Empirically confirmed by instrumenting `evaluate()`:

| Position (white only) | Engine eval |
|---|---:|
| White pawn on **a2** (starting) | **113** |
| a3 | 104 |
| a4 | 99 |
| a5 | 90 |
| a6 | 89 |
| **a7** (one square from promotion) | **90** |

The engine thinks a starting pawn is worth **23 cp more** than a passer on the 7th rank. Pushing `e2-e4` is scored as **-45 cp from white's point of view** in the engine's own evaluator. This is not subtle — it is the opposite of chess.

The king PST is equally broken (or is an endgame-style table mislabeled as MG; either way, the effect is the same):

| White KingMG position (MG-ish pieces on board) | Engine eval |
|---|---:|
| K on **e1** home | +847 |
| K on **g1** castled | **+828** (worse!) |
| K on **e4** marching into the center | **+892** (best!) |

The engine has a **+45 cp bonus** for the middlegame king walking to e4. This is directly visible in `tournament_results.pgn` — e.g. round 1 game 1, `5. Kf1 ?!` played unprovoked by the engine on move 5, which no sane engine would ever do.

This is **the** reason the engine loses to Stockfish. Fix the indexing and/or the king table first; nothing else will matter until it is fixed.

### 1.2 Hard-coded duplicated PST rows (data bugs)

`src/eval/pst.rs`:

- **Bishop MG row 7 is a copy-paste of row 0** (`-29, 4, -82, -37, -25, -42, 7, -8` appears at both `PST_MG[Bishop][0..8]` and `PST_MG[Bishop][56..64]`). The correct PeSTO values are entirely different.
- **Queen MG row 7 is a copy-paste of Rook MG row 7** (`-31, -20, -14, -1, 21, 10, 4, -2`). Nothing to do with the queen.
- **`MATERIAL_MG == MATERIAL_EG == [100, 320, 330, 500, 900, 0]`**. Not tapered at all. The real PeSTO values are `MG=[82,337,365,477,1025]`, `EG=[94,281,297,512,936]`. So the design doc's claim of "PeSTO with tapered material" is false.
- Several knight/bishop/queen rows do not match published PeSTO.

### 1.3 No repetition detection (correctness FR gap)

`grep -r repetition src/` returns nothing. `src/board/mod.rs` has no position history, no stack of prior hashes, and `is_draw_by_fifty()` is the **only** draw check. Consequences:

- **Three-fold repetition is never detected**. The engine cannot find perpetual-check draws, will happily repeat a "winning" position thinking it is making progress, and will lose drawn positions because it does not know they are drawn.
- Combined with the WDL-only tablebase probe (below), the engine can *know* it is winning KRvK but never convert before the 50-move counter fires.

There is no insufficient-material check either.

This is a silent violation of the spirit of FR-03 / UCI conformance: GUIs expect engines to recognize basic draws.

### 1.4 TT mate-score adjustment is missing

`src/search/alphabeta.rs:79-102` and `:209-217` store `best_score as i16` and return TT scores as-is. Mate scores are ply-relative; storing them without adjusting by ply is a classic TT bug that produces:

- Phantom mates (mate-in-N reported when it is really mate-in-M).
- Incorrect cutoffs at transposed positions.
- PV instability on mates across iterations.

Standard fix: on store, add ply to `+mate` scores and subtract from `-mate`; on probe, undo. Not done here.

### 1.5 TT cutoff at the root (ply 0) with no guard

`src/search/alphabeta.rs:80-102` performs a TT cutoff without checking `ply > 0`. At the root that is wrong:

- It silently skips the actual iteration of iterative deepening when the TT already has `depth >= requested_depth` (e.g. after `ucinewgame` with residual entries, or after pondering).
- If the entry is a bound (not Exact), the root can return the bound score without having a fresh `best_move` for this iteration, relying on whatever move the TT happens to still carry.

Root search should always run the move loop.

### 1.6 TT has no replacement policy, no aging

`src/search/tt.rs:102-112` — `store` unconditionally overwrites. There is no depth-preferred replacement, no two-bucket scheme, no generation counter. Over a long game the TT fills with shallow leaf entries and cuts higher-depth useful entries. Expected strength loss: tens of ELO.

### 1.7 Pondering burns the move's clock

`src/search/alphabeta.rs:62-72`: the elapsed-time check is skipped while `pondering=true`, but `state.start_time` is set at `SearchState::new` — i.e. at the start of ponder. On `ponderhit`, `pondering` flips to false, and the very next node-counter tick sees `elapsed = (wall clock including all ponder time)` vs the movetime allocated for the real move, and stops immediately.

Observable effect: a good engine wants to *gain* time by pondering. This engine *loses* its entire move budget if the opponent's move is the predicted one. Either reset `start_time` on `ponderhit` or subtract the ponder duration.

### 1.8 `parse_go` does not wait for the previous search thread

`src/uci.rs:275`: `engine.search_handle = Some(thread::spawn(...))` overwrites the handle without joining. If a previous search is still alive (e.g. during `go ponder` → `go` without an explicit `stop`), two search threads run concurrently against the same TT, and the `engine.stop.store(false, ...)` on line 266 **re-enables the old search** that had stopped. At minimum `stop` should be set, the old handle joined, and only then should the new search start.

---

## 2. Algorithmic strength gaps (why depth 6-7 is all you get)

The search is textbook-bare. Missing everything that turns a toy engine into a 1800+ one:

- **No LMR** (late move reductions). This alone accounts for ~100 ELO at mid depths.
- **No futility/reverse futility pruning**, no razoring, no delta pruning in qsearch.
- **No SEE (Static Exchange Evaluation)** — move ordering is pure MVV-LVA with no losing-capture detection; qsearch explores `QxP?` lines that drop the queen.
- **No check extensions, no singular extensions, no PV extensions.**
- **No history heuristic, no counter-move heuristic, no continuation history.**
- **No internal iterative deepening** (some positions will have no TT move and no ordering at the root of a subtree).
- **Aspiration windows are absent** — every iteration is `[-INFINITY, INFINITY]`.
- **Quiescence does not search quiet promotions** — `generate_captures` skips pushes (`src/movegen/mod.rs:141,215-242`), so a pawn on the 7th rank can sit there quietly and qsearch will never let the side that owns it see the promotion. Classic horizon source.
- **Quiescence does not search checks, does not prune by SEE, has no stand-pat delta pruning.**

### 2.1 Lazy SMP is barely lazy

`src/search/smp.rs:41-65` — helpers only vary `start_depth` by `1 + (i % 3)`. No per-thread skipping of iterations, no aspiration diversity, no move-shuffling, no thread-local killer/history. The verify report itself shows 1 → 4 threads goes from 4.99M to 5.03M NPS: **0.6% scaling**. That is not Lazy SMP, that's four threads doing nearly identical work on the same TT. Useless as implemented.

### 2.2 Move ordering allocates a `Vec` per node

`src/search/ordering.rs:60-72`:
```rust
let mut combined: Vec<(Move, i32)> = moves.as_slice().iter()
    .zip(scores.iter())
    .map(|(&m, &s)| (m, s))
    .collect();
combined.sort_by_key(|&(_, s)| -s);
```
A heap allocation and a full sort at **every interior node**. Should be an in-place sort of the existing `MoveList` (or staged move ordering). Significant hidden NPS cost.

### 2.3 `piece_at` is an O(12) linear scan

`src/board/mod.rs:572-582` iterates all 12 piece bitboards on every call. It is called from move ordering (`pos.piece_at(mv.to_sq())` per move) and `is_capture_or_promotion` inside the search hot loop. A mailbox `[Option<(Color,Piece)>; 64]` kept in sync with the bitboards would eliminate this.

### 2.4 Legal-move generation is copy-make verify

`src/movegen/mod.rs:89-105` — `generate_moves` runs pseudo-legal gen, then clones the `Position` for each move and runs `is_in_check` to filter. Correct but slow. Pinned-piece detection and a check-evasion generator would be roughly 5× faster.

### 2.5 Tablebase is WDL-only, never DTZ

`src/tablebase.rs` has `probe_dtz` but `src/search/alphabeta.rs:105-118` only calls `probe_wdl`. The engine can read "this is a win" but cannot pick the move that converts it — and with no repetition detection (§1.3) it will cycle and draw by the 50-move rule. The search also uses `probe_wdl_after_zeroing`, which is fine for cutoffs, but DTZ should still guide the root move in winning positions.

### 2.6 The UCI `Threads` option does not live-resize the pool

`src/uci.rs:165-169` stores the value on `engine.pool`, but `parse_go` (`:276`) constructs a **new** `ThreadPool::new(num_threads)` inside the spawned closure, using the saved count. Works, but the `ThreadPool` struct is a zero-state holder — there is no persistent worker pool; every `go` spawns all threads fresh. Minor.

---

## 3. Correctness / protocol issues

- `src/uci.rs:66` — `println!("id author Gemini CLI");`. Ship-blocker if you meant to claim authorship.
- `src/uci.rs:82-142` — `parse_position` has no clock reset / rollback on `make_move` failure. A move that fails to parse is silently skipped while later moves are still applied — the resulting position can desync from the GUI. Should either reject the whole `position` command or propagate the error.
- `src/uci.rs:194-288` — `parse_go` has no reply (`bestmove 0000` or similar) when there are zero legal moves and no book. In a checkmate/stalemate position a GUI may hang waiting.
- `src/time.rs:41-52` — no hard vs soft time split; no stability extension; no network/overhead buffer beyond a 50 ms safety margin. On a GUI over pipes or a slow machine this will flag on increment games.
- `src/search/mod.rs:65-107` — `iterative_deepening` uses `limits.movetime` as the only stop condition but does not implement `nodes` or `depth`-only mode separately from the thread start/stop. `params.nodes` is parsed in UCI but never enforced anywhere in search.
- `src/book.rs` — the decoded polyglot move is handed to `println!("bestmove {}", ...)` without legality verification against the current move list (`src/uci.rs:196-202`). A corrupt / wrong-hash book will cause the engine to emit an illegal move, which some GUIs will take as a loss on an illegal move.
- `src/board/mod.rs` — no `make_move` validation; invalid `flags()` values silently unreachable. Not a bug today but fragile.
- `Move::NONE == Move(0)` and `Move::new(Square(0), Square(0), 0, 0)` also equal `Move(0)`. Legal move generation never produces `from==to`, so it currently works, but the sentinel is not distinguishable by construction.
- `tests/perft_tests.rs` only runs startpos to depth 4 in the integration test suite; the acceptance criterion in `requirements.md` FR-02 specifies **depth 6 = 119,060,324**. The verify report claims "PASS" against a criterion that is not actually checked in CI.
- `src/search/alphabeta.rs:74-77` — only `is_draw_by_fifty` is checked. No `ply == 0` guard, so at the root a draw-by-50 is reported as `score=0` but no `bestmove` is found; iterative deepening then falls back to `moves[0]` which is arbitrary.
- `src/search/alphabeta.rs:222-253` — `quiescence` does not increment `ply` and has no ply bound. In pathological positions (e.g. long forced recaptures) it can recurse deeply; `stack overflow` rather than `horizon` is the failure mode. It also does not check `stop`, so `stop` during a tactical qsearch can be delayed indefinitely.
- `src/search/tt.rs:121-130` — `hashfull` samples only the first 1000 entries linearly; with a populated TT and small hash this over-reports, with a large hash it under-reports (occupancy is not uniform over the first 1000 slots at all).

---

## 4. The self-congratulatory documentation is wrong

`.sddw/chess_engine/verify/report.md` claims:
- FR-02 "PASS" based on `perft_tests.rs passes ... up to depth 4+" — the FR acceptance criterion is depth 6.
- FR-04 "PASS" — the evaluation is empirically broken; `evaluate(startpos + e2e4)` gives white a **negative** score.
- FR-11 "PASS" — the NPS numbers in the same report (`4.99M → 5.03M` for 1 → 4 threads) themselves demonstrate **no** SMP scaling.

`.sddw/chess_engine/review/review.md` calls the implementation "excellent quality" and "likely meets or exceeds the ~1800 ELO target". Tournament PGN in the repo shows **18 / 50 points against Stockfish skill 3**, which is not an ~1800 ELO engine by any measure. Both documents should be treated as unreliable.

---

## 5. What to fix, in priority order

1. **Fix PST indexing** — either flip the lookup (`let eval_sq = if color == Color::White { (sq.0 ^ 56) as usize } else { sq.0 as usize };`) **or** rewrite the tables in LERF. Verify with `eval(startpos after e2e4)` > 0 and `eval(pawn on a7) > eval(pawn on a2)`.
2. **Fix/replace the king MG table** — the current one is an endgame-style center-seeking table; the middlegame needs high values on the home rank / castled squares.
3. **Fix the two duplicated PST rows** (Bishop MG row 7, Queen MG row 7). Use real PeSTO values. While at it, use real tapered MG/EG material.
4. **Add repetition detection** — a small `Vec<u64>` stack of hashes in `Position` (cleared on irreversible moves, grown on make_move), `is_draw_by_repetition(ply)` in search, and probe it before the TT.
5. **Adjust TT mate scores on store/probe by `ply`.**
6. **Guard the TT cutoff with `ply > 0`.** Root must always run its move loop.
7. **Fix pondering clock**: reset `state.start_time` on `ponderhit`, or track ponder duration separately.
8. **Join the old search thread in `parse_go`** before spawning a new one.
9. **`id author`**, and legality-check book moves before emitting them.
10. Only after the above — strength work: LMR, SEE-based ordering, aspiration windows, real Lazy SMP diversification, history heuristic, staged move ordering, a mailbox `piece_at`, qsearch check & promotion inclusion. Each of these is worth real ELO; none of them matters while §1.1 is unfixed.

The engine is not at 1800 ELO, it is not "excellent quality", and it is not "fit for purpose". It is a correct-move-generator with a broken evaluator bolted on, and a verification pipeline that is not actually verifying the requirements it claims to verify.
