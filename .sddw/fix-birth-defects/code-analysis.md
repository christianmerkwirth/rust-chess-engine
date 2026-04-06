## Codebase Analysis — fix-birth-defects

Scope: patterns, interfaces, flows, and conventions relevant to the FR-01..FR-19 bug-fix surface in `.sddw/fix-birth-defects/requirements.md`. Grounded in the actual source tree; sites are cited `file:line`.

### Relevant Patterns

- **Copy-make search model**: `alphabeta::search` (`src/search/alphabeta.rs:161-164`) does `next_pos = pos.clone(); next_pos.make_move(mv)` per move. There is no unmake. `movegen::generate_moves` (`src/movegen/mod.rs:97-103`) also verifies legality by clone-and-check. Consequence for FR-05: any repetition history must live on `Position` itself (travels with clones), not in a search-side stack.

- **Tapered PeSTO eval with LERF bitboards**: `src/eval/mod.rs:8-52` computes material + PST MG/EG and interpolates by phase (`compute_phase`, 0..24). Squares are LERF (`src/types.rs:1-7`, `a1=0, h8=63`). The `sq.0 ^ 56` flip at `src/eval/mod.rs:28-32` is inverted relative to the physical PST layout in `src/eval/pst.rs`; FR-01/FR-02 fix is in that flip **or** in the table layout, not both.

- **Ply-relative mate scores with wide cutoff band**: `MATE_SCORE = 30000`, `INFINITY = 32000` (`src/search/alphabeta.rs:11-12`); leaves return `-MATE_SCORE + ply` (`:149`); ID treats `|score| > MATE_SCORE - 1000` as mate (`src/search/mod.rs:105`); UCI printer uses the same band (`src/uci.rs:291-294`). FR-07 must reuse these constants.

- **TT as lock-free XOR-trick array**: `src/search/tt.rs:52-112` uses `[TTEntry { word0, word1 }]` with `word0 = hash ^ packed_data` and unconditional overwrite in `store`. Packed layout: `score(16) | move(16) | depth(8) | flag(2)` — 38 bits used, 26 free. FR-09 (depth-preferred + aging) has room to add a generation field without widening to two u64s.

- **Atomic stop/ponder/nodes threaded through `SearchState`**: `src/search/alphabeta.rs:18-45` carries `&AtomicBool stop`, `&AtomicBool pondering`, `&AtomicU64 nodes`. Polled every 2048 nodes at `:58-72`. Quiescence (`:222-253`) does **not** poll. FR-16 (nodes limit) and FR-17 (qsearch stop + ply bound) plug into the same cadence.

- **Two start-times in ID vs SearchState**: `iterative_deepening` captures its own `start_time = Instant::now()` at `src/search/mod.rs:49` for logging; `SearchState::new` captures a separate `start_time` at `src/search/alphabeta.rs:40` and that is the one consulted by the movetime check at `:65`. FR-10 (ponder clock) hits the `SearchState` one.

- **Lazy SMP helpers spawn a fresh `ThreadPool` inside the outer search closure**: `src/uci.rs:275-288` spawns one outer worker per `go`; inside, `ThreadPool::new(num_threads).search(...)` spawns `num_threads - 1` helpers and runs ID on the calling thread (`src/search/smp.rs:37-86`). All helpers share the same `Arc<AtomicBool> stop` and the same TT. FR-11 (join on new `go`) acts on the **outer** `engine.search_handle` in `uci.rs:275`, not on anything inside `smp.rs`.

- **Polyglot book probe emits `bestmove` directly**: `src/uci.rs:196-202` — if `book.probe(&pos)` returns `Some(mv)`, `println!("bestmove ...")` and return. `PolyglotBook::probe` (`src/book.rs:49-96`) decodes via `decode_polyglot_move(raw, pos)` with no cross-check against `generate_moves(pos)`. FR-15 is a one-site legality filter at the call in `uci.rs`, not inside `book.rs`.

- **`position` command applies moves incrementally without rollback**: `src/uci.rs:111-142` resets `engine.pos` eagerly then walks `moves` tokens, silently skipping unparseable ones while later tokens still apply — position desyncs. FR-13 is staging into a temporary `Position` and committing only on full success.

### Key Interfaces

- **`Position` (`src/board/mod.rs:21-30`)** — `{ piece_bb[12], color_bb[2], side, castling, en_passant, halfmove_clock, fullmove_number, hash }`. Surface: `startpos()`, `from_fen`, `to_fen`, `make_move(&mut self, Move)` (`:350`), `make_null_move` (`:616`), `is_in_check(&self, Color)` (`:528`), `piece_at(Square) -> Option<(Color,Piece)>` (`:572`, O(12) scan — **don't touch** per FR-P1), `pieces(color,piece) -> Bitboard`, `occupancy()`, `occupancy_color(color)`, `side_to_move()`, `castling_rights()`, `en_passant_square()`, `hash() -> u64`, `is_draw_by_fifty() -> bool` (`:633`), `compute_hash()`, `parse_move(uci)`. FR-05 adds a repetition history field + `is_draw_by_repetition()`; FR-06 adds `is_draw_by_insufficient_material()` as a sibling to `is_draw_by_fifty`.

- **`generate_moves(pos) -> MoveList` / `generate_captures(pos) -> MoveList`** (`src/movegen/mod.rs:89-123`). `MoveList` is a fixed `[Move; 256]` stack array (`:6-9`). `pub fn perft(pos, depth: u32) -> u64` at `:389` — directly callable from the FR-18 integration test.

- **`TranspositionTable` (`src/search/tt.rs:62-131`)** — `new(size_mb)`, `probe(hash) -> Option<TTData>`, `store(hash, TTData)`, `clear()`, `hashfull()`. `TTData = { depth: i8, score: i16, flag: TTFlag, best_move: Move }`; packed by `TTData::pack/unpack` (`:19-49`) into one u64. 26 packed bits free for a generation/age counter (FR-09). `TTFlag = Exact | LowerBound | UpperBound`. FR-07 wraps `probe`/`store` with ply-aware score translation; FR-09 adds a generation field + `TT::new_search()` called from `ucinewgame` at `uci.rs:75`.

- **`SearchState<'a>` (`src/search/alphabeta.rs:18-45`)** — `{ killers, nodes: &AtomicU64, stop: &AtomicBool, pondering: &AtomicBool, start_time: Instant, movetime: Option<u64>, tablebase }`. Created in `iterative_deepening` at `src/search/mod.rs:50`. FR-10 resets `start_time` on the `pondering: true → false` transition detected inside the poll block at `alphabeta.rs:63-71`. FR-16 adds `nodes_limit: Option<u64>` checked in the same poll block. FR-17 adds a `qply` arg to `quiescence` and a `MAX_QPLY` constant.

- **`iterative_deepening` (`src/search/mod.rs:37-115`)** — `(pos, tt, limits, stop, pondering, nodes, tablebase, start_depth, info_callback) -> SearchResult`. `SearchLimits { depth, movetime, infinite, ponder }` — **no `nodes` field today**. FR-16 adds `limits.nodes` and wires it through `time::allocate_time` (`src/time.rs:18-55`), which currently discards `params.nodes`.

- **`ThreadPool::search` (`src/search/smp.rs:27-86`)** — `(&self, pos, Arc<TT>, &SearchLimits, Arc<AtomicBool> stop, Arc<AtomicBool> pondering, Option<Arc<SyzygyTablebase>>, info_callback)`. Spawns `num_threads - 1` helpers; main thread runs ID; sets `stop` and joins helpers at the end. Called from the outer thread spawned in `uci.rs:275`. FR-11 acts on the **outer** handle.

- **`Engine` (`src/uci.rs:15-49`)** — `{ pos, tt: Arc<TT>, stop: Arc<AtomicBool>, pondering: Arc<AtomicBool>, search_handle: Option<JoinHandle<()>>, book, tablebase, pool }`. `search_handle` is the single outer worker thread. FR-11: before assigning `engine.search_handle = Some(spawn(...))` at `uci.rs:275`, if the old handle is `Some`, set `engine.stop` + clear `pondering`, `.join()` it, then reset `stop` to false.

- **`PolyglotBook::probe(pos) -> Option<Move>`** (`src/book.rs:49-96`) — decodes Polyglot bits via `decode_polyglot_move(raw, pos)`. No legality check. FR-15 filter: at the caller in `uci.rs:196-202`, `if generate_moves(&pos).into_iter().any(|m| m == mv) { emit; return; }` — fall through to search otherwise.

- **UCI dispatch (`src/uci.rs:51-109`)** — match on first token. `"uci"` → id strings (FR-12 site: `:66`). `"ucinewgame"` → `tt.clear()` (FR-09 co-site: also call `tt.new_search()`). `"position"` → `parse_position` (FR-13). `"go"` → `parse_go` (FR-11, FR-14, FR-15, FR-16). `"ponderhit"` → clears pondering (FR-10 co-site). FR-14: before spawning the search, pre-flight `generate_moves(&engine.pos).is_empty()` → `println!("bestmove 0000")` and return.

- **`GoParams` / `allocate_time`** (`src/time.rs:5-55`) — holds `wtime/btime/winc/binc/movestogo/depth/nodes/movetime/infinite/ponder`; `allocate_time` returns `SearchLimits` but currently drops `params.nodes` entirely. FR-16 wiring passes through here.

- **Test harness**: `tests/perft_tests.rs` uses `chess_engine::movegen::{magics::init, perft}` + `Position::{startpos, from_fen}`. `tests/uci_tests.rs` spawns `./target/debug/chess_engine` and pipes UCI via `UciEngine::{send, wait_for}`. FR-18 perft test slots into `perft_tests.rs`; FR-19 eval-sanity tests slot into the existing `#[cfg(test)] mod tests` at `src/eval/mod.rs:72-132`; FR-12/FR-13/FR-14 UCI tests slot into `uci_tests.rs`.

### Existing Flows

- **UCI `uci` handshake** (`src/uci.rs:64-73`): print `id name ChessEngine` → `id author Gemini CLI` → option lines → `uciok`. **FR-12:** change `:66` to `id author Christian Merkwirth`.

- **`position startpos moves ...`** (`src/uci.rs:111-142`): detect `startpos`/`fen`, reset `engine.pos` eagerly, walk `moves` tokens calling `engine.pos.parse_move(tok); if Some { engine.pos.make_move(mv) }`. Unknown tokens silently no-op while later tokens keep applying. **FR-13:** stage into a local `Position`, apply all moves to the copy, commit to `engine.pos` only on full success.

- **`go` → search thread spawn** (`src/uci.rs:194-288`): (1) book probe → direct `bestmove`; (2) parse `GoParams`; (3) `allocate_time` → `SearchLimits`; (4) reset `stop`, set `pondering = params.ponder`; (5) clone context into closure; (6) `engine.search_handle = Some(thread::spawn(...))` — **overwrites any prior handle without joining**. **FR-11:** between (4) and (5), if `search_handle.is_some()` → `stop.store(true)`, clear `pondering`, take+join the handle, then reset `stop` to false. **FR-14:** between (1) and (2), if legal moves empty → `println!("bestmove 0000")` and return. **FR-15:** in (1), wrap `book.probe(&pos)` with a legality check; fall through otherwise. **FR-16:** in (3), pass `params.nodes` into `SearchLimits`.

- **`go ponder` → `ponderhit` time flow** (`src/uci.rs:92-94`; `src/search/alphabeta.rs:40, 63-71`; `src/search/mod.rs:49-50`): `parse_go` sets `pondering = true`; `SearchState::new` captures `start_time = Instant::now()` **at ponder start**; while `pondering`, movetime check at `alphabeta.rs:63-71` is skipped; on `ponderhit` the flag flips to false and the next poll tick sees `elapsed = full ponder wall clock` vs `movetime` and stops immediately. **FR-10:** track `was_pondering: bool` on `SearchState`; inside the poll block, on the `true → false` transition reset `state.start_time = Instant::now()`.

- **Iterative deepening main loop** (`src/search/mod.rs:65-108`): for `depth in start_depth..=max_depth` call `search(pos, state, tt, depth, 0, -INF, INF)`; on `stop`, probe TT for best move and break; else update best, compute PV, info callback; break on mate. **FR-08 interaction:** the root call always enters `search` with `ply == 0`; the TT cutoff fires before the move loop, short-circuiting the iteration without refreshing `best_move`. **FR-16:** `max_depth = limits.depth.unwrap_or(100)` already handles `go depth`; add `limits.nodes` early-exit check at the start of each iteration and in the poll block.

- **Alpha-beta node** (`src/search/alphabeta.rs:47-220`): (1) bump nodes, poll stop/time every 2048 nodes; (2) `is_draw_by_fifty return 0`; (3) TT probe → cut unconditionally on depth-sufficient hit (**FR-08 bug**); (4) tablebase WDL probe; (5) NMP; (6) `depth <= 0 → quiescence`; (7) generate legal moves; empty → `-MATE_SCORE + ply` or `0`; (8) `order_moves`; (9) PVS loop; (10) TT store using raw `best_score as i16` (**FR-07 bug**: ply not folded). **FR-05:** insert `if ply > 0 && pos.is_draw_by_repetition() return 0` before TT probe, and **skip TT store** on repetition nodes. **FR-06:** co-located `if pos.is_draw_by_insufficient_material() return 0`. **FR-07:** wrap probe at `:85` with `score_from_tt(score, ply)`, wrap store at `:213` with `score_to_tt(best_score, ply)`. **FR-08:** guard the `depth >= depth` cut with `ply > 0`.

- **TT store → probe lifecycle** (`src/search/tt.rs:85-112`): `store` packs and unconditionally overwrites via XOR trick; `probe` reads `word1, word0`, checks `word0 ^ word1 == hash`, returns `Some` or `None`. **FR-09:** add `generation: u8` to `TTData` (reuse the 26 free packed bits). TT holds its own `AtomicU8` generation; `store` replacement rule: overwrite if `old.generation != current` OR `new.depth >= old.depth`. Add `TT::new_search(&self)` that fetch-adds the generation; call from `ucinewgame` (`uci.rs:75`).

- **Quiescence** (`src/search/alphabeta.rs:222-253`): stand-pat → `generate_captures` → `order_moves` (no killers) → recurse. **No stop check, no ply counter, no ply bound.** **FR-17:** add `qply: usize` arg + `MAX_QPLY` constant (e.g. 64); thread `qply + 1` into the recursive call; at the limit return `stand_pat`; reuse the `nodes & 0x7FF == 0` cadence to poll `state.stop` and return `alpha`.

- **`make_move` + hash update** (`src/board/mod.rs:350-524`): XOR out old castling/EP → branch on `mv.flags()` → reset/increment `halfmove_clock` (reset on capture, pawn move, promotion, EP; incremented otherwise including castling at `:480`) → update castling rights → XOR in new castling/EP → flip side → bump fullmove if black. **FR-05:** add a `history: Vec<u64>` on `Position`. On every `make_move`, if the new `halfmove_clock == 0` **or** castling rights changed, clear history; otherwise push the pre-mutation hash. The existing `halfmove_clock = 0` branches (`:373, :375, :411, :446`) are the authoritative irreversible-move markers; verified by `test_halfmove_clock_*` tests at `:1031-1069`.

- **Perft test entry** (`tests/perft_tests.rs:4-14`): `magics::init()` → `Position::startpos()` → `perft(&pos, N)` asserts. Current ceiling is depth 4. **FR-18:** add `test_perft_startpos_depth_6` asserting `perft(&pos, 6) == 119_060_324`. Slow in debug; gate on release or accept the runtime.

- **Eval test entry**: `#[cfg(test)] mod tests` at `src/eval/mod.rs:72-132` already has `test_evaluate_startpos`, `test_evaluate_symmetry`, etc. **FR-19:** add `eval_e2e4_improves_white`, `eval_a7_beats_a2`, `king_mg_prefers_back_rank`, `pst_bishop_row7_matches_pesto`, `material_mg_ne_eg` in the same module.

### Conventions

- **LERF square indexing throughout** (`src/types.rs:1-7`, `src/board/mod.rs:182, 195`, `src/movegen/mod.rs`): `Square(rank * 8 + file)`, `a1 = 0`, `h8 = 63`. `src/eval/pst.rs:4` claims LERF in a comment but the bytes are laid out visual/BERF (row 0 = rank 8). Pick one source-of-truth for the PST files and make the `eval_sq` flip at `src/eval/mod.rs:28-32` match. Do not mix conventions.

- **Copy-make, not make-unmake**: `next_pos = pos.clone(); next_pos.make_move(mv)` is universal (`src/search/alphabeta.rs:163-164, :239-240`; `src/movegen/mod.rs:97-99, :115-117`; `src/search/mod.rs:142`). FR-05 repetition history must live on `Position` itself; do not introduce a separate undo stack — it will diverge across clones.

- **Incremental Zobrist with XOR in/out pairs** (`src/board/mod.rs:357-361, :385-386, :512-519`); `compute_hash()` at `:638-657` is the from-scratch reference, exercised by `test_zobrist_incremental_after_castling` at `:1204`. FR-05 must push either the pre- or post-mutation hash **consistently** (convention is pre-mutation; write the chosen rule into the design).

- **Atomics for cross-thread state**: `Arc<AtomicBool>` for stop/pondering, `Arc<AtomicU64>` for the shared node counter (`src/search/alphabeta.rs:8-9, :56`; `src/search/smp.rs:38`). FR-11 reuses the same `Arc<AtomicBool>`; FR-16's nodes limit reads `state.nodes` (already shared).

- **Poll cadence `nodes & 0x7FF == 0`** (`src/search/alphabeta.rs:58-72`). FR-16 and FR-17 ride the same cadence — don't introduce a second constant. For qsearch, `state.nodes.fetch_add` at `:223` already increments, so `nodes & 0x7FF == 0` works there too.

- **Tests live next to the code** (`#[cfg(test)] mod tests` in `src/types.rs`, `src/board/mod.rs:667+`, `src/eval/mod.rs:72+`, `src/search/alphabeta.rs:265+`, `src/search/tt.rs:133+`, `src/search/ordering.rs:75+`, `src/search/smp.rs:89+`). `tests/` holds only cross-crate integration. FR-19 → `src/eval/mod.rs mod tests`; FR-18 → `tests/perft_tests.rs`; FR-12/FR-13/FR-14 → `tests/uci_tests.rs`.

- **Magics init is a test prerequisite**: every test that exercises movegen begins with `crate::movegen::magics::init();` (e.g. `src/search/alphabeta.rs:272, :291`, `src/search/ordering.rs:84, :95`, `tests/perft_tests.rs:6`). Any new test for FR-05..FR-08 that generates moves must do the same.

- **`MoveList` is fixed `[Move; 256]`, not `Vec`** (`src/movegen/mod.rs:6-9`). `src/search/ordering.rs:60-72` violates this by building a `Vec<(Move, i32)>` per node — out of scope per FR-P1 (staged move ordering is deferred). Do not "fix" this as a drive-by.

- **`piece_at` is an O(12) linear scan** (`src/board/mod.rs:572-582`). Mailbox cache is out of scope per FR-P1. Do not add one in this feature.

- **UCI output style**: plain `println!("...")` + `io::stdout().flush().unwrap()` at dispatch boundaries (`src/uci.rs:102, :199, :286`). Non-fatal errors use `eprintln!("info string ...")` (`:176, :186`). FR-13 rejects bad `position` input silently via `info string`, not `panic!`.

- **`id name` / `id author` are unconditional string literals at the UCI handshake** — no env lookup, no build-time substitution. FR-12 hardcodes `Christian Merkwirth` at `src/uci.rs:66`.

- **Ply-relative mate constants**: `MATE_SCORE = 30000`, mate band `|score| > MATE_SCORE - 1000`, matches `src/uci.rs:291-294` `(30000 - |score| + 1) / 2`. FR-07's TT helpers must reuse these:
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

- **Comments are sparse and load-bearing**: module-doc comments on `Position` (`src/board/mod.rs:16-19`) and tricky functions (`src/search/tt.rs:52-60, :89-94, :107-109`). Per-line commentary is avoided. Don't flood fix diffs with narration; comment only where the logic isn't self-evident.

- **No `unwrap()` on recoverable paths in UCI**: `src/uci.rs` uses `if let Some/Ok` and silent no-ops; `io::stdout().flush().unwrap()` is the only allowed unwrap. FR-13 failure mode is silent rejection, not panic.

- **`.sddw/fix-birth-defects/` is the scope root** per requirements (`SHALL NOT modify .sddw/chess_engine/requirements.md`). All artifacts for this feature — code-analysis, design, implement, verify — live under `.sddw/fix-birth-defects/`.
