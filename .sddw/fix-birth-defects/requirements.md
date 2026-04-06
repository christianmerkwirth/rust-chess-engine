# Requirements: fix-birth-defects

## 1. Project

- Path: `.`

Motivating document: `.sddw/chess_engine/extended-review.md`. This feature addresses the critical bugs in §1 and the protocol/correctness issues in §3 of that review. The §2 strength gaps (LMR, SEE, aspiration windows, staged move ordering, real Lazy SMP diversification, history heuristic, mailbox `piece_at`, qsearch promotions/checks) are explicitly deferred to a follow-up feature.

## 2. Purpose

Fix the critical evaluation, search, transposition-table, time-management, and UCI protocol bugs documented in `extended-review.md` (§1 and §3) so that the engine evaluates positions correctly, detects standard draws, reports mates consistently, and spends its move clock on the right move. Strength work (LMR, SEE, aspiration windows, staged move ordering) is explicitly deferred to a follow-up feature.

## 3. User Stories

- As a chess player using the engine through a UCI GUI, I want the engine to push pawns toward promotion, castle, and keep the king safe in the middlegame, so that I am playing against actual chess rather than an engine that walks its king to e4 on move 5.
- As the engine maintainer, I want draws (three-fold repetition, insufficient material) and mate scores to be reported consistently across the search and the transposition table, so that the engine claims draws when they exist, converts winning endgames, and reports stable mate distances.
- As the author, I want the engine's UCI identity to credit me accurately, so that downstream tournaments and logs attribute the work to Christian Merkwirth.

## 4. Functional Requirements

### Evaluation (§1.1, §1.2)

- FR-01: The engine SHALL index piece-square tables so that, from white's point of view, `evaluate(startpos after e2e4) > evaluate(startpos)`.
- FR-02: The engine SHALL use a middlegame king PST where, on a middlegame-phase board, `evaluate(white king on e1) > evaluate(white king on e4)` AND `evaluate(white king on g1) > evaluate(white king on e4)`.
- FR-03: The engine SHALL replace the duplicated Bishop MG row 7 and Queen MG row 7 in `src/eval/pst.rs` with the correct published PeSTO values.
- FR-04: The engine SHALL use distinct tapered MG/EG material values such that `MATERIAL_MG != MATERIAL_EG` for pawn, knight, bishop, rook, and queen.

### Search correctness (§1.3–§1.8)

- FR-05: The engine SHALL detect three-fold repetition as a draw during search by tracking a per-position hash history that grows on `make_move` and is cleared at irreversible moves (captures, pawn moves, castling, rights changes).
- FR-06: The engine SHALL detect insufficient-material draws for at least KvK, KBvK, KNvK, and KBvKB with same-colored bishops.
- FR-07: The engine SHALL adjust mate scores by `ply` on both TT store and TT probe, so that a mate score stored at one ply is reinterpreted correctly when probed at a different ply.
- FR-08: The engine SHALL NOT take a TT cutoff at the root (`ply == 0`); the root move loop SHALL always execute at least once per iteration.
- FR-09: The engine SHALL implement a depth-preferred TT replacement policy with a generation/age counter that advances on `ucinewgame`, so that deeper entries and recent entries are preferred over shallow stale ones.
- FR-10: The engine SHALL preserve its move-clock budget across `ponderhit` by either resetting `start_time` on `ponderhit` or subtracting the ponder duration from elapsed time.
- FR-11: The engine SHALL stop and join any in-flight search thread before spawning a new one in response to a subsequent `go` command, so that at most one search thread is concurrently active against the TT.

### UCI and protocol (§3)

- FR-12: The engine SHALL advertise `id author Christian Merkwirth` in response to the UCI `uci` command.
- FR-13: The engine SHALL apply a `position` command atomically: if any move in the provided move list fails, the internal position SHALL NOT be left partially updated.
- FR-14: The engine SHALL emit `bestmove 0000` when `go` is issued in a position with zero legal moves and no book move, so that UCI GUIs do not hang.
- FR-15: The engine SHALL verify any book move against the current legal move list and fall back to normal search if the book move is not legal.
- FR-16: The engine SHALL enforce `go nodes N` and `go depth N` limits inside the search loop, not only via external wall-clock timing.
- FR-17: The engine's quiescence search SHALL check the stop flag on a bounded cadence AND SHALL enforce a maximum qsearch ply to prevent stack overflow on pathological tactical lines.

### Verification (agreed additions)

- FR-18: The test suite SHALL include a perft test that asserts `perft(startpos, 6) == 119_060_324`, and this test SHALL run as part of `cargo test`.
- FR-19: The test suite SHALL include eval-sanity regression tests that assert each of: white eval increases on `e2e4`; a white pawn on a7 evaluates higher than a white pawn on a2; with a middlegame board, white king on e1 and g1 both beat white king on e4; `PST_MG[Bishop]` row 7 matches published PeSTO; `MATERIAL_MG != MATERIAL_EG`.

### Prohibitions

- FR-P1: This feature SHALL NOT implement any §2 strength work — specifically no LMR, no SEE, no aspiration windows, no staged move ordering, no history heuristic, no counter-move heuristic, no real Lazy SMP diversification, no mailbox `piece_at` cache, and no qsearch promotion/check inclusion. These are deferred to a follow-up feature so that the before/after strength signal for this feature reflects bug fixes, not feature additions.
- FR-P2: This feature SHALL NOT introduce NNUE or neural-network evaluation (inherited from the original `chess_engine` requirements FR-12).

## 5. Acceptance Criteria

### FR-01: PST indexing direction

**Happy path:**
- GIVEN `Position::startpos()`
- WHEN `evaluate(&pos)` is called from white's point of view
- AND a child position is constructed after `e2e4`
- THEN `evaluate(child)` SHALL be strictly greater than `evaluate(startpos)` (SHALL)

**Boundary:**
- GIVEN an empty board with a single white pawn on a7 vs a single white pawn on a2
- WHEN both positions are evaluated from white's POV
- THEN the a7 evaluation SHALL be strictly greater than the a2 evaluation (SHALL)

### FR-02: Middlegame king table

- GIVEN a middlegame position with most pieces on the board (phase close to 0)
- WHEN the white king is placed on e1, g1, and e4 in turn
- THEN `evaluate(K on e1) > evaluate(K on e4)` (SHALL)
- AND `evaluate(K on g1) > evaluate(K on e4)` (SHALL)

### FR-03: Duplicated PST rows

- GIVEN `src/eval/pst.rs`
- WHEN the PST_MG tables are compared against published PeSTO values
- THEN `PST_MG[Bishop]` row 7 (squares 56..64) SHALL equal the PeSTO bishop MG row 7 (SHALL)
- AND `PST_MG[Queen]` row 7 (squares 56..64) SHALL equal the PeSTO queen MG row 7 (SHALL)
- AND neither row SHALL be a verbatim copy of another row or another piece's row (SHALL NOT)

### FR-04: Tapered material

- GIVEN `MATERIAL_MG` and `MATERIAL_EG` arrays
- THEN for pawn, knight, bishop, rook, and queen, `MATERIAL_MG[p] != MATERIAL_EG[p]` (SHALL)
- AND for a position with exactly one minor piece, the tapered material score at phase = 0 (full material on the board) SHALL differ from the tapered material score at phase = 24 (bare kings + minor) (SHALL)

### FR-05: Three-fold repetition

**Happy path:**
- GIVEN a sequence of knight moves that returns to the same position three times (e.g. `Ng1-f3 Ng8-f6 Nf3-g1 Nf6-g8 Ng1-f3 Ng8-f6 Nf3-g1 Nf6-g8`)
- WHEN the search visits the third occurrence during alpha-beta
- THEN the node SHALL be scored 0 (draw) (SHALL)
- AND the TT SHALL NOT be probed or updated for that node (SHALL NOT)

**Boundary:**
- GIVEN an irreversible move (capture, pawn move, castling, or rights change)
- WHEN `make_move` applies it
- THEN the repetition history SHALL be cleared (or its baseline advanced) at that ply so pre-irreversible-move occurrences do not count (SHALL)

### FR-06: Insufficient material

- GIVEN any of KvK, KBvK, KNvK, or KBvKB with same-colored bishops
- WHEN `is_draw_by_insufficient_material(&pos)` is called
- THEN it SHALL return true (SHALL)
- AND the search SHALL return score 0 for that node without descending (SHALL)

### FR-07: TT mate-score ply adjustment

- GIVEN a TT entry stored with `score = MATE - 2` at search `ply = 5`
- WHEN the same position is probed at `ply = 3`
- THEN the returned score SHALL reflect the mate distance from ply 3's perspective (SHALL)
- AND a mate-in-N result found via a TT hit SHALL match the mate distance of a fresh search of the same depth to the same position (SHALL)

### FR-08: Root TT cutoff guard

- GIVEN a TT entry for the root position with `depth >= requested_depth`
- WHEN `alpha_beta` is called with `ply == 0`
- THEN the root move loop SHALL execute at least once for this iteration (SHALL)
- AND a `best_move` SHALL be produced by this iteration (SHALL)

### FR-09: TT replacement and aging

- GIVEN a TT slot currently occupied by an entry at depth D1
- WHEN a new entry for the same index at depth D2 > D1 is stored
- THEN the new entry SHALL replace the old one (SHALL)

- GIVEN a TT slot occupied by an entry from generation G-1
- WHEN a new entry from generation G at any depth is stored
- THEN the stale entry SHALL be replaceable (SHALL)

- GIVEN `ucinewgame` is received
- WHEN it is processed
- THEN the TT generation counter SHALL advance (SHALL)

### FR-10: Ponder clock preservation

- GIVEN `go ponder` starts at wall-clock time `t0` and runs for 2000 ms
- AND `ponderhit` arrives with an effective `movetime` of 1000 ms
- WHEN the search proceeds after `ponderhit`
- THEN the elapsed-time check SHALL be measured from the instant of `ponderhit`, not from `t0` (SHALL)
- AND the engine SHALL NOT stop within the first few ms after `ponderhit` on account of burned ponder time (SHALL NOT)

### FR-11: Search thread join on new `go`

- GIVEN an in-flight search thread (for example, started by `go ponder`)
- WHEN a new `go` command is received
- THEN the engine SHALL signal the old search to stop (SHALL)
- AND SHALL join the old search thread before spawning a new one (SHALL)
- AND at any instant no more than one search thread SHALL be running against the shared TT (SHALL)

### FR-12: UCI author identity

- GIVEN the engine is started
- WHEN `uci` is sent
- THEN the engine SHALL emit exactly one line matching `^id author Christian Merkwirth$` (SHALL)

### FR-13: `position` atomicity

- GIVEN the engine is in the startpos
- WHEN `position startpos moves e2e4 <illegal>` is sent where `<illegal>` cannot be parsed or is not legal
- THEN the internal position SHALL remain at startpos (SHALL)
- AND SHALL NOT reflect the partial application of `e2e4` (SHALL NOT)
- AND the engine SHALL log or silently reject the command without panicking (SHALL)

### FR-14: `bestmove 0000` in terminal positions

- GIVEN a checkmate or stalemate position loaded via `position fen ...`
- WHEN `go` (or `go movetime N`) is sent
- THEN the engine SHALL respond with `bestmove 0000` within the budget (SHALL)
- AND SHALL NOT hang or produce no output (SHALL NOT)

### FR-15: Book move legality

- GIVEN a Polyglot book lookup returns a `Move` that does not appear in the current legal move list (e.g. corrupt / wrong-hash book)
- WHEN the engine is deciding what to play
- THEN the engine SHALL discard the book move (SHALL)
- AND SHALL fall back to normal search (SHALL)
- AND SHALL NOT emit an illegal bestmove (SHALL NOT)

### FR-16: `go nodes` and `go depth` enforcement

- GIVEN `go nodes 10000`
- WHEN the search runs
- THEN the search SHALL stop after approximately 10000 nodes (within a bounded overshoot tolerance for the current iteration) (SHALL)

- GIVEN `go depth 5`
- WHEN the search runs
- THEN iterative deepening SHALL NOT begin an iteration deeper than depth 5 (SHALL NOT)

### FR-17: Quiescence safety

- GIVEN a `stop` signal arrives while quiescence is searching a long forced-recapture line
- WHEN quiescence proceeds
- THEN it SHALL observe `stop` within a bounded number of qnodes (SHALL)

- GIVEN a qsearch ply bound (e.g. 64)
- WHEN quiescence recurses
- THEN it SHALL NOT recurse deeper than the bound (SHALL NOT)

### FR-18: Perft depth 6 in CI

- GIVEN the integration test suite
- WHEN `cargo test` is run
- THEN a test (for example `perft_startpos_depth_6`) SHALL run and assert the node count equals `119_060_324` (SHALL)
- AND SHALL pass (SHALL)

### FR-19: Eval sanity regression tests

- GIVEN the test suite
- WHEN `cargo test` is run
- THEN the following tests SHALL exist and pass (SHALL):
  - `eval_e2e4_improves_white`
  - `eval_a7_beats_a2`
  - `king_mg_prefers_back_rank`
  - `pst_bishop_row7_matches_pesto`
  - `material_mg_ne_eg`

### FR-P1: No §2 strength work

- GIVEN a diff of this feature's changes against the pre-feature branch point
- WHEN reviewed
- THEN no new LMR, SEE, aspiration-window, staged-ordering, history-heuristic, counter-move-heuristic, mailbox `piece_at`, or qsearch promotion/check code SHALL have been added (SHALL NOT)

### FR-P2: No NNUE

- GIVEN the evaluation pipeline
- WHEN any position is evaluated
- THEN no neural-network inference SHALL be performed (SHALL NOT)

## 6. Constraints

### In Scope

- Fix of PST orientation, middlegame king table, duplicated PST rows, and tapered MG/EG material (§1.1, §1.2)
- Three-fold repetition and insufficient-material draw detection (§1.3)
- TT mate-score ply adjustment on store and probe (§1.4)
- TT root-cutoff guard (§1.5)
- TT depth-preferred replacement with generation/age counter (§1.6)
- Ponder-clock preservation on `ponderhit` (§1.7)
- Search-thread join on new `go` (§1.8)
- UCI `id author Christian Merkwirth` (§3)
- `position` command atomicity (§3)
- `bestmove 0000` in terminal positions (§3)
- Book-move legality verification before emitting (§3)
- `go nodes` and `go depth` enforcement inside the search loop (§3)
- Quiescence stop-flag check and ply bound (§3)
- Perft depth 6 test in CI (agreed addition)
- Eval-sanity regression tests (agreed addition)
- Post-fix rematch vs Stockfish skill 3 via existing `scripts/stockfish_match.*` pipeline, recorded in the verify report

### Out of Scope

- LMR (late move reductions) — §2 strength work, deferred to a follow-up feature
- SEE-based move ordering and SEE-based qsearch pruning — §2 strength work, deferred
- Aspiration windows — §2 strength work, deferred
- Staged move ordering, history heuristic, counter-move heuristic, continuation history — §2 strength work, deferred
- Real Lazy SMP diversification (per-thread iteration skipping, thread-local killers, aspiration diversity, move shuffling) — §2 strength work, deferred
- Mailbox `[Option<(Color,Piece)>; 64]` cache for `piece_at` — §2 NPS work, deferred
- Qsearch check extensions and quiet-promotion inclusion — §2 strength work, deferred (horizon improvement)
- Replacing the copy-make legal-move generator with a pinned-piece / check-evasion generator — §2 NPS work, deferred
- DTZ-guided tablebase root-move selection — §2 strength work, deferred
- NNUE / neural evaluation — permanently out of scope per original `chess_engine` requirements FR-12
- Chess960 / Fischer Random — not targeted by the review
- Live-resizable persistent thread pool — minor, not strength-critical

### Prohibitions

- SHALL NOT implement any §2 strength feature in this feature's branch — the before/after strength signal should reflect bug fixes, not feature additions; mixing the two makes it impossible to attribute regressions
- SHALL NOT silently weaken or disable existing features (opening book, Syzygy tablebases, pondering, Lazy SMP) as a shortcut to making a bug go away — the review identifies real defects in these subsystems; disabling them is not a fix
- SHALL NOT regress perft depth 6 (`119_060_324` from startpos) — move generation is the one subsystem the review confirmed as correct to depth 6
- SHALL NOT modify `.sddw/chess_engine/requirements.md` or the rest of the original `chess_engine` design docs — this feature lives under `.sddw/fix-birth-defects/` and leaves the historical record alone
- SHALL NOT ship a verify report claiming PASS without the FR-01..FR-19 acceptance criteria actually being exercised by automated tests or a recorded command transcript — motivated directly by §4 of the review, in which the existing verify report claimed PASS against criteria that were never run
- SHALL NOT change the `id author` line to anything other than `Christian Merkwirth` — no "Gemini CLI", no "Claude Code", no co-authors in the UCI identity

### Testing Approach

- TDD — write failing tests first, then implement fixes to make them pass. For each §1 bug with a concrete numerical reproducer (FR-01 eval sign, FR-02 king table, FR-03 duplicated rows, FR-04 tapered material, FR-05 three-fold, FR-06 insufficient material, FR-07 TT mate scores, FR-08 root cutoff, FR-10 ponder clock, FR-11 thread join, FR-14 bestmove 0000, FR-15 book legality, FR-16 go nodes/depth, FR-17 quiescence bound, FR-18 perft-6, FR-19 eval sanity), the failing test SHALL be committed before the fix. This both proves the bug existed and guards against regression. The §3 items without a clean reproducer (FR-12 id author string, FR-13 parse_position atomicity) MAY be covered by assertion-style tests introduced alongside the fix.

