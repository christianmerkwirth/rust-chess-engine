## Codebase Analysis — push-up-elo

Scope: patterns, interfaces, flows, and conventions relevant to FR-01..FR-14 in `.sddw/push-up-elo/requirements.md`. Grounded in the actual source tree at HEAD (post `fix-birth-defects` merge); sites are cited `file:line`. Companion to `.sddw/fix-birth-defects/code-analysis.md` — facts already documented there are not re-stated unless they bind a push-up-elo decision.

### Relevant Patterns

- **Tapered eval (MG/EG interpolation)**: `src/eval/mod.rs:8-54` accumulates `mg[2]` and `eg[2]` per side, then interpolates by phase: `(mg_score * phase + eg_score * (24 - phase)) / 24`. Phase counter at `:58-71`: P=0, N=B=1, R=2, Q=4, capped at 24. **Implication for FR-07/FR-08/FR-09:** every new term must contribute to **both** `mg`/`eg` accumulators so the existing taper applies for free. King-safety endgame taper is achieved by setting eg = 0 on the king-safety contribution, not by an early-out.

- **Material + PST is the only existing eval term**: `src/eval/mod.rs:35-36` does `mg[c] += MATERIAL_MG[p] + PST_MG[p][eval_sq]` per piece in a single pass over the 12 (color, piece) cells. There is no per-piece-type loop separation today. **Implication for FR-07 (mobility):** the natural shape is one extra branch inside the existing per-piece loop keyed on `piece in {Knight, Bishop, Rook, Queen}` that calls the matching `magics::*_attacks(sq, occ)` and adds `(MOB_MG[p][n], MOB_EG[p][n])`. No new outer pass needed.

- **PST flip via `sq.0 ^ 56` for white** (`src/eval/mod.rs:27-33`): PST tables are BERF (`src/eval/pst.rs:4`, row 0 = rank 8), so white squares get xor-flipped, black squares used directly. The fix-birth-defects feature ratified this combination. **Implication:** any new PST-style table (king-safety attacker weights, mobility tables) MUST follow the same `eval_sq = if white { sq.0 ^ 56 } else { sq.0 }` convention, OR live in a colour-symmetric form. Re-introducing a flip mismatch will silently cost Elo and skip detection by `test_evaluate_symmetry`.

- **Per-piece pseudo-attack functions exposed by `magics`**: `magics::knight_attacks(sq)`, `bishop_attacks(sq, occ)`, `rook_attacks(sq, occ)`, `queen_attacks(sq, occ)`, `king_attacks(sq)`, `pawn_attacks(color, sq)` (`src/movegen/magics.rs:170-196`). Returns raw attack bitboards before any friendly-occupancy mask. **Implication for FR-07:** `mobility = (attack_bb & !us_occ).count()`. **Implication for FR-08:** counting enemy attackers on the king zone reuses these directly — no new movegen.

- **Bitboard file/rank constants exist** (`src/bitboard.rs:10-26`): `FILE_A..FILE_H`, `RANK_1..RANK_8`. **Implication for FR-09 (pawn structure) and FR-11 (rook on open file):** doubled / isolated / open / semi-open are pure bitwise operations on these constants — no new lookup tables. **Note:** there are no pre-built `ADJACENT_FILES`, passed-pawn front-span, or king-zone bitboards. Passed-pawn detection needs a 64-entry per-colour front-span table, derived once.

- **Move ordering allocates a `Vec` per node** (`src/search/ordering.rs:32-72`): `order_moves` builds a `[i32; 256]` scores array, materialises a `Vec<(Move, i32)>`, sorts by `-s`, and copies back. Slot priority: PV (10M) → captures MVV-LVA (1M+) → promotion (900k) → killer1 (800k) → killer2 (700k) → 0 (everything else). **Implication for FR-03:** history scores plug into the trailing `else` branch with range `[1, 600_000]` so they sort below killer2 but above unscored quiets. The `Vec` allocation is a known wart deferred per fix-birth-defects FR-P1 — **do not "fix" it** as part of push-up-elo.

- **Killer table is a fixed `[[Move; 2]; 256]` indexed by ply** (`src/search/ordering.rs:7-30`). **Implication for FR-03:** history is **not** ply-indexed — it's `(side_to_move, from, to)` global. Use a separate `HistoryTable` struct on `SearchState` next to `killers`, not nested inside `KillerTable`.

- **`SearchState` is the carrier for cross-call mutable search context** (`src/search/alphabeta.rs:41-73`). It already holds `killers, nodes, stop, pondering, start_time, movetime, nodes_limit, tablebase, was_pondering`. **Implication for FR-03 / FR-06:** any new mutable per-search state goes here (history table, optionally an LMR reduction table). Aspiration window state lives in `iterative_deepening`, not on `SearchState`, because it's only consumed by the ID layer.

- **PVS (principal variation search) loop in alpha-beta** (`src/search/alphabeta.rs:206-243`): first move full window `[-beta, -alpha]`, subsequent moves null window `[-alpha-1, -alpha]` with full re-search on `score > alpha && score < beta`. **Implication for FR-06 (LMR):** the LMR reduction integrates **inside** the existing null-window branch — reduce, search, and on `score > alpha` re-search at full depth (and possibly full window). Do not bypass PVS.

- **NMP already implemented at `src/search/alphabeta.rs:166-185`** with `R=3`, gated on `depth >= 3 && !in_check && ply > 0 && non_pawn_pieces`. **Implication for FR-02 (RFP):** RFP must be gated by the same predicates plus `!is_pv` and a near-mate guard, and structurally lives **between** the TT cut block (`:147`) and NMP (`:166`). RFP and NMP are independent — both can fire on the same node.

- **Single-pass leaf detection: `if depth <= 0 → quiescence`** (`src/search/alphabeta.rs:187`). **Implication for FR-04 (check extension):** the `+1` extension lives **before** this branch is reached at the child level. Concretely: compute `extension = if next_pos.is_in_check(next_pos.side_to_move()) { 1 } else { 0 }` per child, then call children with `depth - 1 + extension`. Do not mutate `depth` at the top of the parent.

- **Quiescence: stand-pat + captures-only; no delta yet** (`src/search/alphabeta.rs:270-340`). FR-17 (qply bound, MAX_QPLY=64) and the time/stop poll cadence are already in. **Implication for FR-05:** delta-pruning predicate slots into the move loop **before** `next_pos.clone()` to save the clone cost. Victim value comes from `pos.piece_at(mv.to_sq())` (or `Pawn` for en-passant). Reuse `PIECE_VALUES`.

- **`measure-elo` tooling**: `just measure-elo 20 5 20` runs `uv run scripts/measure_elo.py --games 20 --sf-skill 5 --concurrency 20` (`justfile:69-71`), which builds a `cutechess-cli` invocation pairing `./target/release/chess_engine` against `stockfish` at `tc=10+0.1`, parses `Elo difference: <val> +/- <margin>` from cutechess output (`scripts/measure_elo.py:62`), and prints `FINAL ESTIMATED ELO DIFF: <val> +/- <margin>` on success (`scripts/measure_elo.py:129`). **Implication for FR-12/FR-13:** the gating loop greps the `FINAL ESTIMATED ELO DIFF:` line. No machine-readable JSON; one-line grep is the contract. The 20-game sample is statistically very noisy (margin typically wider than the delta) — this is the rule the requirements lock in regardless.

- **`measure_elo.py` has a latent bug**: `import math` lives inside `main()` (`scripts/measure_elo.py:116`) but `run_match()` uses `math.isnan(e)` at `:71`. The first parsed `Elo difference:` line raises `NameError` and silently leaves `elo_diff = None`. **Implication:** push-up-elo will hit this on the first gauntlet. Either move `import math` to module top as a small prerequisite fix, or grep cutechess output ourselves in the gating loop (out-of-band of the script's `FINAL ESTIMATED ELO DIFF:` line). Document the chosen path in `/sddw:design`.

- **No run-log infrastructure exists**: `.sddw/push-up-elo/run-log.md` (called out in FR-12 / requirements §6) is a NEW file the implement step writes by hand after each gauntlet. Format below in Conventions.

### Key Interfaces

- **`evaluate(pos: &Position) -> i32`** (`src/eval/mod.rs:8-54`) — returns side-to-move centipawn score. Single function, no internal sub-term split. **FR-07..FR-11** add term computation inside the same function (or extracted helpers in `src/eval/`) but the public surface stays one function. All new terms accumulate into the existing `mg`/`eg` arrays before the taper at `:47` so `test_evaluate_symmetry` keeps passing.

- **`compute_phase(pos) -> i32`** (`src/eval/mod.rs:58-71`) — 0..24. **FR-08:** new terms taper to zero in endgame by setting their eg value to 0 (preferred — lets the existing taper do the work). Don't introduce a second phase function.

- **`Position` accessors** (`src/board/mod.rs:538-608`): `pieces(color, piece) -> Bitboard`, `occupancy() -> Bitboard`, `occupancy_color(color) -> Bitboard`, `side_to_move() -> Color`, `is_in_check(color) -> bool`, `piece_at(sq) -> Option<(Color, Piece)>`. **All read-only consumption.** Do not add new methods to `Position` for push-up-elo — eval helpers are free functions in `src/eval/` taking `&Position`.

- **`magics::knight_attacks(sq) / bishop_attacks(sq, occ) / rook_attacks(sq, occ) / queen_attacks(sq, occ) / king_attacks(sq) / pawn_attacks(color, sq) -> Bitboard`** (`src/movegen/magics.rs:170-196`). **FR-07:** mobility = `(attack_bb & !us_occ).count()`. **FR-08:** intersect with king-zone bb to count attackers.

- **`Bitboard` constants and ops** (`src/bitboard.rs`): `FILE_A..FILE_H`, `RANK_1..RANK_8`, `count() -> u32`, `is_empty() -> bool`, `is_set(sq) -> bool`, `pop_lsb() -> Square`, iterator. **FR-09:**
  - Doubled: `(white_pawns & FILE_X).count() >= 2` per file
  - Isolated: `white_pawns & FILE_X != 0 && white_pawns & (FILE_{X-1} | FILE_{X+1}) == 0`
  - Passed: per-square front-span bb (file ahead + two adjacent files); pawn passed iff `enemy_pawns & front_span == 0`. The 64-entry `PASSED_FRONT_SPAN_WHITE/BLACK: [u64; 64]` is best built once at module init or as a `const` derived via a small `const fn`.
  
  **FR-11:**
  - Open file (per side, per file X): `(white_pawns | black_pawns) & FILE_X == 0`
  - Semi-open (white): `white_pawns & FILE_X == 0 && black_pawns & FILE_X != 0`

- **`order_moves(moves: &mut MoveList, pv_move: Move, killers: &[Move; 2], pos: &Position)`** (`src/search/ordering.rs:32-73`). **FR-03 wiring:** signature must extend to `order_moves(moves, pv_move, killers, history, pos)` where `history: &HistoryTable`. The history score is consulted in the trailing `else` branch (currently `score = 0`) for quiet, non-PV, non-killer moves. Capture/promotion/killer slots stay untouched. **All call sites update**: `src/search/alphabeta.rs:200` (alpha-beta) and `:318` (quiescence — pass an empty/default history; quiescence has no history feedback loop).

- **`HistoryTable` (NEW)** — proposed shape, lives in `src/search/ordering.rs`:
  ```rust
  pub struct HistoryTable { table: [[[i16; 64]; 64]; 2] } // [side][from][to]
  impl HistoryTable {
      pub fn new() -> Self { /* zero */ }
      pub fn record(&mut self, side: Color, from: Square, to: Square, depth: i32);
      pub fn score(&self, side: Color, from: Square, to: Square) -> i32;
      pub fn age(&mut self); // halve all entries between IDs (optional)
  }
  ```
  Bonus formula: `bonus = (depth * depth).min(MAX_BONUS)`. Saturate via `saturating_add` clamped to `[i16::MIN/2, i16::MAX/2]` to avoid overflow per FR-03 unit-level acceptance criterion. Calling site: `src/search/alphabeta.rs:236-239` — when a quiet move causes a beta cutoff, update killers **and** record history.

- **`SearchState<'a>` extension** (`src/search/alphabeta.rs:41-73`): add `history: HistoryTable` field for FR-03. If FR-06 needs a precomputed log-based reduction table, either add `lmr_table: [[i32; 64]; 64]` here, or live as a module-level `OnceLock` const since it doesn't depend on runtime state. Recommend the const route — fewer SearchState fields, no per-search init cost.

- **`SearchLimits`** (`src/search/mod.rs:15-22`): `{ depth, movetime, nodes, infinite, ponder }`. **No changes for push-up-elo** — none of FR-01..FR-11 add UCI options or limit knobs (FR-17 prohibits it).

- **`iterative_deepening`** (`src/search/mod.rs:39-133`): the only ID-layer change is FR-01. Wrap the `search(..., -INFINITY, INFINITY)` call at `:74` in an aspiration loop using `prev_score` (carry from the previous iteration's `best_score`). Mate-score guard: if `|prev_score| > MATE_SCORE - 1000`, fall back to the full `[-INF, INF]` window for the next iteration. **No FR other than FR-01 touches this function.**

- **`search` and `quiescence`** (`src/search/alphabeta.rs:75-266` and `:270-340`): all of FR-02, FR-03, FR-04, FR-05, FR-06 splice into these two functions inline. Do not extract them into separate functions — they need access to all of `state`, `tt`, the move loop counter `i`, and the moves array, and inlining is the established convention here (cf. how NMP at `:166-185` is inlined).

- **`PIECE_VALUES`** (`src/search/ordering.rs:5`): `[100, 320, 330, 500, 900, 20000]`. **FR-05 (delta pruning) needs them in `alphabeta.rs`.** Recommend making the constant `pub` and importing — single source of truth. These are search piece values, distinct from `MATERIAL_MG/EG` in `pst.rs` (`[82,337,365,477,1025,0]` / `[94,281,297,512,936,0]`) which are tapered eval values.

- **`measure_elo.py` CLI surface**: `--engine PATH`, `--engine-args STR`, `--stockfish PATH`, `--sf-skill INT`, `--tc STR`, `--games INT`, `--concurrency INT`. **Output contract:** the line `FINAL ESTIMATED ELO DIFF: <signed_float> +/- <margin>` always appears on success. The gating loop greps for that line, not the per-checkpoint `Elo difference:` lines (cutechess emits multiple progress updates).

- **Test entry points** (unchanged from fix-birth-defects):
  - **Eval tests** → `src/eval/mod.rs` `mod tests` (`:73-194`). FR-07..FR-11 unit tests live here. `test_evaluate_symmetry` is the colour-mirror gate.
  - **Move-ordering tests** → `src/search/ordering.rs` `mod tests` (`:75-130`). FR-03 history table tests live here.
  - **Search-behaviour tests** → `src/search/alphabeta.rs` `mod tests` (`:352-416`). FR-04 (check extension behavioural) and FR-05 (delta predicate) unit tests here.
  - **Magics init prerequisite** — every test that exercises movegen calls `crate::movegen::magics::init();`. Eval tests that build positions via `Position::from_fen` should call it too as a safe default.

### Existing Flows

- **Iterative deepening main loop** (`src/search/mod.rs:39-133`):
  1. `start_time = Instant::now()`
  2. `state = SearchState::new(stop, pondering, nodes, limits.movetime, limits.nodes)`; `state.tablebase = tablebase`
  3. Seed `best_move` from `generate_moves(pos)[0]`
  4. `for depth in start_depth..=max_depth`:
     - node-limit pre-check (`:67-72`)
     - `score = search(pos, &mut state, tt, depth, 0, -INFINITY, INFINITY)` ← **FR-01 wraps this in the aspiration loop**, using `prev_score` from the previous iteration. Mate-score guard: skip aspiration when `|prev_score| > MATE_SCORE - 1000`.
     - On `state.stop`, recover `best_move` from TT and break
     - `best_score = score`; refresh `best_move` from TT probe; compute PV; emit info callback
     - Break on mate (`|best_score| > MATE_SCORE - 1000`)
  5. Final info callback + return `SearchResult { best_move, score, ponder_move }`
  
  **FR-01 only.** Add `prev_score: Option<i32> = None` outside the loop, set to `Some(best_score)` at the bottom of each successful iteration.

- **Alpha-beta node** (`src/search/alphabeta.rs:75-266`):
  1. `nodes += 1`; on `nodes & 0x7FF == 0`, poll stop / movetime / nodes-limit (existing)
  2. Draw checks: `is_draw_by_fifty()` always; `is_draw_by_repetition() / is_draw_by_insufficient_material()` when `ply > 0` (existing)
  3. **TT probe** with `score_from_tt(score, ply)`; cut on depth-sufficient hit when `ply > 0` (existing)
  4. **Tablebase probe** if `≤ 6` pieces (existing)
  5. **Reverse Futility Pruning (FR-02 — NEW)** — splice **between (4) and (6)**:
     - guards: `!in_check && ply > 0 && depth <= RFP_MAX_DEPTH (e.g. 6) && abs(beta) < MATE_SCORE - 1000 && !is_pv` (where `!is_pv ≈ beta - alpha == 1` since this engine has no explicit pv flag)
     - body: `let static_eval = evaluate(pos); if static_eval - margin(depth) >= beta { return static_eval }` (or `beta`)
     - margin: small linear, e.g. `margin(d) = 80 * d`
  6. **Null Move Pruning** at `depth >= 3 && !in_check && ply > 0 && non_pawn_pieces` (existing)
  7. `if depth <= 0 { return quiescence(pos, state, alpha, beta, 0) }` (existing)
  8. `generate_moves(pos)`; checkmate / stalemate handling (existing)
  9. `order_moves(...)` (existing) — **FR-03 wires the history table into this call**
  10. `for i in 0..moves.len()`:
      - `next_pos = pos.clone(); next_pos.make_move(mv)` (existing)
      - **Check extension (FR-04 — NEW)**: `let ext = if next_pos.is_in_check(next_pos.side_to_move()) { 1 } else { 0 };` Apply at child call sites: replace `depth - 1` with `depth - 1 + ext`.
      - **Late Move Reductions (FR-06 — NEW)**: for `i >= LMR_MIN_MOVE_INDEX (e.g. 4)`, `!is_pv`, `!in_check (parent)`, `!is_capture_or_promotion(mv)`, `!gives_check (next_pos)`, compute `r = lmr_table[depth][i]`. Search reduced; if `score > alpha`, re-search at full depth. Composes with PVS as the **outer** reduction step.
      - PVS as today: first move full window, others null window, fail-high re-search (existing — `:212-222`)
      - On beta cutoff, store killer (existing) **and update history (FR-03)**: `if !is_capture_or_promotion(mv) { history.record(side, mv.from, mv.to, depth) }`
  11. **TT store** with `score_to_tt(best_score, ply)` (existing)

- **Quiescence** (`src/search/alphabeta.rs:270-340`):
  1. `nodes += 1`; poll cadence on `nodes & 0x7FF == 0` (existing)
  2. `qply >= MAX_QPLY → return evaluate(pos)` (existing — FR-17)
  3. `stand_pat = evaluate(pos)`; `if stand_pat >= beta return stand_pat`; raise alpha (existing)
  4. `generate_captures(pos)`; `order_moves(...)` with default history (existing — signature change for FR-03 must accept a no-op history here)
  5. `for i in 0..moves.len()`:
     - **Delta pruning (FR-05 — NEW)** **before** `next_pos.clone()`:
       ```
       let victim_value = if mv.is_en_passant() {
           PIECE_VALUES[Pawn]
       } else {
           PIECE_VALUES[pos.piece_at(mv.to_sq()).map(|(_, p)| p as usize).unwrap_or(0)]
       };
       if !mv.is_promotion() && stand_pat + victim_value + DELTA_MARGIN < alpha { continue; }
       ```
     - clone, recurse, update alpha/beta (existing)
  6. Return `alpha`

- **`evaluate` flow** (`src/eval/mod.rs:8-54`):
  1. Initialise `mg = [0, 0]`, `eg = [0, 0]`
  2. For each `(color, piece)` in the 12-cell loop, for each square in `pos.pieces(color, piece)`:
     - `eval_sq = if white { sq.0 ^ 56 } else { sq.0 }`
     - `mg[c] += MATERIAL_MG[p] + PST_MG[p][eval_sq]`
     - `eg[c] += MATERIAL_EG[p] + PST_EG[p][eval_sq]`
  3. `mg_score = mg[W] - mg[B]`; `eg_score = eg[W] - eg[B]`
  4. `phase = compute_phase(pos)`
  5. `score = (mg_score * phase + eg_score * (24 - phase)) / 24`
  6. Negate if black to move
  
  **FR-07 (mobility)** splices into step 2's inner loop: for `piece in {Knight, Bishop, Rook, Queen}`, compute `mob = (piece_attacks(piece, sq, occ) & !us_occ).count() as usize`, then `mg[c] += MOB_MG[piece][mob]; eg[c] += MOB_EG[piece][mob]`.
  
  **FR-09 (pawn structure)**: best computed **after** the main loop, once per side: extract `pos.pieces(side, Pawn)`, scan files for doubled, scan adjacent-file masks for isolated, scan front-spans for passed. Add to `mg[c]/eg[c]`.
  
  **FR-10 (bishop pair)**: post-loop: `if pos.pieces(side, Bishop).count() >= 2 { mg[side] += BISHOP_PAIR_MG; eg[side] += BISHOP_PAIR_EG; }`.
  
  **FR-11 (rook on file)**: fold into step 2's `Rook` branch *or* a post-loop pass: per rook square, classify file (Open / SemiOpen / Closed), add bonus to `mg[c]/eg[c]`.
  
  **FR-08 (king safety)**: separate post-loop helper. For each side: build `king_zone = magics::king_attacks(king_sq) | Bitboard::from_square(king_sq)`; per enemy attacker piece, count those whose attack bb intersects the king zone; weight by attacker piece type into `attack_units`; add `pawn_shield` penalty for missing/advanced shield pawns on the king's file ± 1; subtract from `mg[us]` (eg contribution = 0 to taper). Highest-complexity new term — extract into a `king_safety(pos, side) -> (i32, i32)` helper.

- **Move ordering flow** (`src/search/ordering.rs:32-73`):
  1. `scores = [0i32; 256]`
  2. For each move, score by priority: PV → capture MVV-LVA → en-passant → promotion → killer1 → killer2 → 0
  3. Build `Vec<(Move, i32)>`, sort by `-s`, copy back
  
  **FR-03 splice:** in step 2, after the killer2 branch, add `else { score = HISTORY_BASE + history.score(side, from, to) }` clamped to `[1, 600_000]` so it lands strictly below `killers[1] = 700_000` and strictly above unscored = 0.

- **Gauntlet flow** (`just measure-elo 20 5 20`):
  1. `cargo build --release` (via the `release` recipe)
  2. `uv run scripts/measure_elo.py --games 20 --sf-skill 5 --concurrency 20`
  3. Spawns `cutechess-cli` pairing `./target/release/chess_engine` vs `stockfish` at `tc=10+0.1`, 20 games (10 rounds × 2 with `-repeat`), 20 concurrent
  4. cutechess prints periodic `Elo difference: ...` lines as games complete
  5. Script greps each line, updates `elo_diff` / `error_margin`
  6. After cutechess exits, script prints `FINAL ESTIMATED ELO DIFF: <signed> +/- <margin>`
  
  **FR-12/FR-13/FR-14 loop** (manual or scripted):
  1. Measure baseline at `HEAD` → record in `.sddw/push-up-elo/run-log.md`
  2. Apply patch N (single commit on `feat/push-up-elo`)
  3. `cargo test --all` — must pass (FR-16 gate)
  4. Run `just measure-elo 20 5 20` → parse final elo
  5. If `final > baseline`: append `kept` row; baseline ← final; HEAD advances
  6. If `final ≤ baseline`: append `reverted` row; `git reset --hard HEAD^`; baseline unchanged
  7. Repeat for next FR

### Conventions

**Inherited from `.sddw/fix-birth-defects/code-analysis.md` (still authoritative):**

- **LERF square indexing** (`src/types.rs:1-7`): `Square(rank * 8 + file)`, `a1 = 0`, `h8 = 63`. Any new bitboard work uses LERF.
- **PST tables are BERF** (`src/eval/pst.rs:4`): row 0 = rank 8. White squares get `^ 56` flip; black squares used directly. Don't flip both, don't flip neither.
- **Copy-make**: `next_pos = pos.clone(); next_pos.make_move(mv)` everywhere. New search-side state lives on `SearchState`, not on `Position` or as thread-locals.
- **Atomics for cross-thread state**: `Arc<AtomicBool>` for stop/pondering, `Arc<AtomicU64>` for nodes. Aspiration windows must check `state.stop` after each re-search or risk infinite loops on widening.
- **Poll cadence `nodes & 0x7FF == 0`** in both alpha-beta and quiescence. No second cadence.
- **Tests live next to the code** in `#[cfg(test)] mod tests`. `tests/` is reserved for cross-crate / UCI integration.
- **Magics init prerequisite**: every test that exercises movegen begins with `crate::movegen::magics::init();`. New eval tests should call it too as a safe default.
- **Ply-relative mate constants**: `MATE_SCORE = 30000`, mate band `|score| > MATE_SCORE - 1000`. `score_to_tt`/`score_from_tt` already exist (`src/search/alphabeta.rs:16-37`). Aspiration bounds and RFP guards reuse these — do not introduce a new mate threshold.
- **`MoveList` is fixed `[Move; 256]`, not `Vec`** (`src/movegen/mod.rs:6-9`). The `Vec` allocation in `order_moves` is a known wart deferred per fix-birth-defects FR-P1 — **do not "fix" it as a drive-by**.
- **`piece_at` is O(12) linear scan** (`src/board/mod.rs:582`). For FR-05, calling `pos.piece_at(mv.to_sq())` once per quiescence move is acceptable — the cost is dwarfed by the `next_pos.clone()` we save by pruning.
- **Comments are sparse and load-bearing**: module-doc comments and tricky-function comments only. Don't flood diffs with narration.
- **No `unwrap()` on recoverable paths** in UCI; only `io::stdout().flush().unwrap()` is allowed.

**New conventions specific to push-up-elo:**

- **Tapered eval shape**: every new eval term (FR-07..FR-11) MUST contribute to **both** `mg[c]` and `eg[c]` accumulators in `src/eval/mod.rs`. Setting eg = 0 (e.g. king safety) is the canonical "tapered to zero in endgame". **Do NOT** add a separate `if compute_phase(pos) > X` short-circuit — let the existing taper at `:47` do the work.

- **Eval term symmetry**: `test_evaluate_symmetry` (`src/eval/mod.rs:108-120`) is the colour-mirror gate. Every new term MUST evaluate identically (and oppositely) for colour-mirrored positions. If your term references file/rank, use white-relative ranks for white and black-relative ranks for black (e.g. via `if white { rank } else { 7 - rank }`). Add a per-term symmetry test in addition to the global one.

- **Magic numbers cited or tagged**: every weight/threshold in a new eval or search term must be either:
  - cited inline with a literature source (e.g. `// Stockfish king-safety attacker weights, evaluate.cpp:1234` or `// Chess Programming Wiki: passed pawn`), or
  - marked `// TODO: tune` if the value is a guess.
  
  Per requirements §6 prohibitions. Do not merge a patch with bare unexplained constants.

- **One FR per commit**: each of FR-01..FR-11 lands as a single commit on `feat/push-up-elo`. **No bundling.** Per FR-12..FR-14, the gauntlet gate operates on the most recent commit, and `git reset --hard HEAD^` is the documented revert path. Bundling makes a partial revert impossible.

- **Branch hygiene**: all work on `feat/push-up-elo` (or similar). Per requirements §6, do not commit directly to `master`. Final merge happens after the day's run completes and the cumulative Elo is in the run-log.

- **Run-log format** (`.sddw/push-up-elo/run-log.md`, NEW): markdown table with one row per gauntlet:
  ```
  | # | patch  | baseline_elo  | post_elo      | delta | margin | verdict | commit |
  |---|--------|---------------|---------------|-------|--------|---------|--------|
  | 0 | (base) | —             | -180.0 ± 30.0 | —     | 30.0   | —       | <sha>  |
  | 1 | FR-01  | -180.0 ± 30.0 | -150.0 ± 28.0 | +30   | 28.0   | kept    | <sha>  |
  | 2 | FR-02  | -150.0 ± 28.0 | -160.0 ± 29.0 | -10   | 29.0   | reverted| —      |
  | 3 | FR-03  | -150.0 ± 28.0 | -120.0 ± 27.0 | +30   | 27.0   | kept    | <sha>  |
  ```
  Per FR-13, baseline is **re-measured at HEAD** before each patch (so it reflects the cumulative effect of previously kept patches). The "baseline" column is the most recent re-measurement, not necessarily the previous row's `post_elo`.

- **`cargo test --all` is a gate before every gauntlet** (FR-16). If a patch breaks tests, fix tests or revert; do **not** mark a test `#[ignore]` to "make it pass".

- **UCI surface frozen** (FR-17): no `setoption` additions, removals, or renames. The `uci` handshake output stays a superset. No new `info string` lines for tuning constants — keep them as Rust `const`s.

- **Selective TDD scope**: write unit tests *first* for the deterministic helpers (history table store/lookup/saturation, passed/isolated/doubled detection, bishop-pair count, rook-on-file predicate, delta-pruning predicate, aspiration re-search loop). For search-flow integrators (RFP, LMR, check ext), the test contract is the existing suite + the gauntlet — don't synthesize fragile "search returns N nodes" assertions.

- **No new `Position` methods**: eval/search changes consume existing `Position` accessors. New helpers go in `src/eval/` or `src/search/` as free functions taking `&Position`.

- **PST file orientation**: if FR-08 introduces a new spatial table (e.g. attacker weight per zone square, or a king-safety lookup keyed by square), it MUST follow the same BERF layout + `^ 56` flip as `src/eval/pst.rs`, **or** use a colour-symmetric formulation (preferred — fewer ways to get the flip wrong).
