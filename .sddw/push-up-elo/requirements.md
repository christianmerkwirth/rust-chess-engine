# Requirements — push-up-elo

## 1. Project

- Path: `.`

---

## 2. Purpose

The engine is missing several well-known classical search and evaluation improvements (aspiration windows, reverse futility pruning, history heuristic, check extensions, delta pruning in quiescence, Late Move Reductions, mobility, king safety, pawn-structure terms, bishop pair, rook on open/semi-open file). This feature closes that gap within a one-day development budget by implementing each as an orthogonal, individually gauntlet-verified patch.

---

## 3. User Stories

- As the engine developer, I want a prioritised, orthogonal menu of classical search/eval improvements with effort estimates and literature grounding, so that I can pick a subset that fits a one-day budget.
- As the engine developer, I want each implemented improvement to be gauntlet-verified against Stockfish via `just measure-elo 20 5 20` before being kept, so that only patches with a positive Elo point estimate remain in the final engine.
- As an end user of the chess engine, I want the compiled binary to play measurably stronger chess after the feature lands, so that matches and analysis sessions reflect the improved search and evaluation.

---

## 4. Functional Requirements

### Search — cheap wins

- FR-01: The search SHALL begin each iterative-deepening iteration with an aspiration window centred on the previous iteration's score and SHALL re-search with a progressively wider window on fail-high or fail-low until a score inside the window is returned.
- FR-02: Non-PV, non-in-check search SHALL apply reverse futility pruning (RFP / static-null-move pruning) at shallow depth: if `static_eval - margin(depth) >= beta`, the node SHALL return the static evaluation (or beta) without further search. RFP SHALL NOT fire at PV nodes, in check, near mate, or at depths above the configured cap.
- FR-03: The search SHALL maintain a butterfly history table of quiet-move beta-cutoffs keyed by `(side_to_move, from_square, to_square)`, and `order_moves` SHALL use this table to rank quiet moves that are neither the PV move nor a killer.
- FR-04: The search SHALL extend its depth by one ply when the side to move is in check (check extension).
- FR-05: Quiescence search SHALL apply delta pruning — a capture SHALL be skipped when `stand_pat + victim_value + delta_margin < alpha` and the move is not a promotion.

### Search — medium

- FR-06: Non-PV, non-in-check search SHALL apply Late Move Reductions (LMR) to quiet moves after the first few, reducing depth by a formula-based amount and re-searching at full depth on fail-high.

### Evaluation enrichment

- FR-07: `evaluate` SHALL add a mobility term for knights, bishops, rooks, and queens based on the number of pseudo-legal attack squares (excluding squares occupied by friendly pieces), tapered between middlegame and endgame.
- FR-08: `evaluate` SHALL add a king-safety term that counts enemy attackers on squares around the friendly king and penalises missing or advanced pawn-shield pawns. The term SHALL be tapered toward zero in the endgame.
- FR-09: `evaluate` SHALL add pawn-structure terms for passed, isolated, and doubled pawns.
- FR-10: `evaluate` SHALL add a bishop-pair bonus that applies when a side has at least two bishops.
- FR-11: `evaluate` SHALL add a rook-on-open-file and rook-on-semi-open-file bonus.

### Process / gating

- FR-12: Each of FR-01 through FR-11 SHALL be introduced as a separate commit and gauntlet-tested via `just measure-elo 20 5 20` (20 games vs Stockfish Skill 5, concurrency 20, default time control). A patch SHALL be KEPT only if its post-patch Elo point estimate exceeds the most recently accepted baseline.
- FR-13: Before each patch's gauntlet, the baseline Elo SHALL be re-measured from the current HEAD so that cumulative gains are attributed to the specific patch under test.
- FR-14: If a patch fails the gating rule in FR-12, the change SHALL be reverted (e.g. `git reset --hard HEAD^` of the patch commit or equivalent) and the loop SHALL continue with the next candidate FR from the previous accepted baseline.

### Prohibitions

- FR-15: The feature SHALL NOT introduce any neural-network evaluation (NNUE, ONNX, libtorch, weights files, or equivalent). Classical terms only.
- FR-16: The feature SHALL NOT weaken, remove, or `#[ignore]` any existing test in `cargo test`, the perft suite, or the UCI integration suite. Every kept patch SHALL leave `cargo test --all` exiting zero.
- FR-17: The feature SHALL NOT change the UCI protocol surface. No existing option SHALL be renamed or removed, and the `uci` handshake output SHALL remain a superset of the pre-feature output so existing GUIs continue to work unmodified.

---

## 5. Acceptance Criteria

### FR-01: Aspiration windows

**Happy path:**
- GIVEN a release build at HEAD before the patch and a baseline Elo measured via `just measure-elo 20 5 20`
- WHEN the aspiration-window patch is applied and `just measure-elo 20 5 20` is run
- THEN `cargo test --all` SHALL exit zero
- AND the post-patch Elo point estimate SHALL exceed the baseline
- AND the patch SHALL be kept

**Failure path:**
- GIVEN the same baseline
- WHEN the gauntlet point estimate is ≤ baseline
- THEN the patch SHALL be reverted and the next candidate SHALL use the pre-patch baseline

**Regression guard:**
- GIVEN `just bench` at depth 8 from startpos
- WHEN run before and after the patch
- THEN the reported best move SHALL remain a reasonable opening move (no NONE / illegal output), and the search SHALL not hang or crash

### FR-02: Reverse futility pruning

**Happy path:** as FR-01 happy path.

**Failure path:** as FR-01 failure path.

**Safety boundary:**
- GIVEN a node where `pos.is_in_check(side_to_move)` is true
- WHEN the search reaches this node
- THEN RFP SHALL NOT fire
- AND the node SHALL be searched normally

**Near-mate boundary:**
- GIVEN a node where `abs(beta) >= MATE_SCORE - 1000`
- WHEN the search reaches this node
- THEN RFP SHALL NOT fire

### FR-03: History heuristic

**Happy path:** as FR-01 happy path.

**Failure path:** as FR-01 failure path.

**Unit-level:**
- GIVEN a fresh history table where move `M = (side, from, to)` has been incremented by several beta-cutoff events
- WHEN `order_moves` is called on a move list containing `M` alongside other quiet non-killer, non-PV moves
- THEN `M` SHALL appear earlier in the ordered list than any other quiet move with a strictly lower history score

**Unit-level (decay):**
- GIVEN a history table near `i16::MAX` for some entries
- WHEN a bonus is added
- THEN values SHALL be clamped or decayed and SHALL NOT overflow

### FR-04: Check extension

**Happy path:** as FR-01 happy path.

**Failure path:** as FR-01 failure path.

**Behavioural:**
- GIVEN a position where the side to move is in check
- WHEN the engine searches at nominal depth `d`
- THEN the effective depth at this node SHALL be `d + 1`

### FR-05: Delta pruning in quiescence

**Happy path:** as FR-01 happy path.

**Failure path:** as FR-01 failure path.

**Unit-level (predicate):**
- GIVEN a quiescence node with `stand_pat + PIECE_VALUES[Queen] + delta_margin < alpha`
- WHEN delta pruning is checked against a pawn-capturing-pawn move
- THEN the move SHALL be skipped

**Safety boundary:**
- GIVEN a promotion capture
- WHEN delta pruning is checked
- THEN the move SHALL NOT be skipped regardless of `stand_pat`

### FR-06: Late Move Reductions

**Happy path:** as FR-01 happy path.

**Failure path:** as FR-01 failure path.

**Safety boundary:**
- GIVEN a PV node or an in-check node
- WHEN a quiet move is searched
- THEN the move SHALL NOT be reduced

**Re-search:**
- GIVEN a reduced-depth search that returns a score > alpha
- WHEN LMR notices the fail-high
- THEN the move SHALL be re-searched at full depth before being accepted

### FR-07: Mobility term

**Unit-level happy path:**
- GIVEN a FEN where a white knight on d4 has 8 attack squares and an otherwise symmetric position
- WHEN `evaluate` is called
- THEN the result SHALL be > 0 from white's perspective
- AND the difference SHALL be attributable to the mobility term

**Symmetry:**
- GIVEN a position and its colour-mirrored counterpart
- WHEN both are evaluated
- THEN the scores SHALL be equal and opposite (extending `test_evaluate_symmetry`)

**Gauntlet:** as FR-01 happy/failure paths.

### FR-08: King safety

**Unit-level happy path:**
- GIVEN a middlegame FEN where black has three attackers on squares around the white king and white has an intact pawn shield on f2/g2/h2 in one variant and a broken shield (pawns on f4/g4/h4) in another
- WHEN both variants are evaluated
- THEN the broken-shield variant SHALL score lower for white than the intact-shield variant

**Endgame taper:**
- GIVEN a KQK endgame with `compute_phase == 0`
- WHEN `evaluate` is called
- THEN the king-safety term's contribution SHALL be zero or near-zero

**Symmetry:** as FR-07 symmetry.

**Gauntlet:** as FR-01 happy/failure paths.

### FR-09: Pawn structure

**Unit-level passed pawn:**
- GIVEN a FEN with a white pawn on a6 and no black pawns on the a, b files ahead of it
- WHEN `evaluate` is called
- THEN the pawn-structure bonus SHALL be positive for white

**Unit-level isolated pawn:**
- GIVEN a FEN with a white pawn on d4 and no white pawns on c or e files
- WHEN `evaluate` is called
- THEN the isolated-pawn penalty SHALL reduce white's score relative to the same position with a c-pawn added

**Unit-level doubled pawns:**
- GIVEN a FEN with two white pawns on the d-file
- WHEN `evaluate` is called
- THEN the doubled-pawn penalty SHALL apply

**Symmetry:** as FR-07 symmetry.

**Gauntlet:** as FR-01 happy/failure paths.

### FR-10: Bishop pair

**Unit-level:**
- GIVEN two FENs identical except one side has `BB` and the other has `BN`
- WHEN both are evaluated
- THEN the side with two bishops SHALL score strictly higher (bishop-pair bonus applied)

**Symmetry:** as FR-07 symmetry.

**Gauntlet:** as FR-01 happy/failure paths.

### FR-11: Rook on open / semi-open file

**Unit-level open file:**
- GIVEN a FEN with a white rook on d1 and no pawns of either colour on the d-file
- WHEN `evaluate` is called
- THEN the rook-on-open-file bonus SHALL apply

**Unit-level semi-open file:**
- GIVEN a FEN with a white rook on d1, no white pawn on the d-file, but a black pawn on d6
- WHEN `evaluate` is called
- THEN the semi-open-file bonus SHALL apply (smaller than the full open-file bonus)

**Symmetry:** as FR-07 symmetry.

**Gauntlet:** as FR-01 happy/failure paths.

### FR-12: Gauntlet gating

**Happy path:**
- GIVEN a patch under test
- WHEN `just measure-elo 20 5 20` completes
- THEN stdout SHALL contain an `Elo difference: <value> +/- <margin>` line
- AND the numeric value SHALL be recorded alongside the patch identifier in a run log (e.g. `.sddw/push-up-elo/run-log.md`)

**Accept:**
- GIVEN a recorded post-patch Elo > the previously accepted baseline
- WHEN the gate evaluates the patch
- THEN the patch SHALL be kept and the baseline SHALL advance to this new value

**Reject:**
- GIVEN a recorded post-patch Elo ≤ baseline
- WHEN the gate evaluates the patch
- THEN the patch SHALL be reverted and the baseline SHALL remain unchanged

### FR-13: Baseline re-measurement

- GIVEN two consecutive accepted patches A then B
- WHEN B is about to be gauntleted
- THEN the baseline used for B's gate SHALL be a freshly measured Elo of HEAD (i.e. including A's effect), not the baseline recorded before A
- AND this fresh baseline SHALL be written to the run log alongside B's measurement

### FR-14: Revert on failure

- GIVEN a patch whose gauntlet run failed the gate (FR-12 reject)
- WHEN the loop moves to the next candidate FR
- THEN the working tree and `HEAD` SHALL be at the state of the last accepted patch (no residual staged files, no leftover eval-term code)
- AND the run log SHALL record the rejected patch with its Elo delta and a `reverted` marker

### FR-15: No NN evaluation

- GIVEN the final diff of the feature compared to the pre-feature `master`
- WHEN inspected
- THEN no dependency in `Cargo.toml` SHALL reference NN frameworks (`tract`, `onnxruntime`, `tch`, `candle`, `burn`, `dfdx`, etc.)
- AND no weights / `.nnue` / `.onnx` / `.safetensors` file SHALL be added
- AND no tensor-valued field SHALL appear in `src/eval/`

### FR-16: Existing tests still pass

- GIVEN any kept patch
- WHEN `cargo test --all` is run
- THEN exit code SHALL be zero
- AND no test SHALL be newly `#[ignore]`d
- AND no test SHALL be deleted without an explicit note in the commit message

### FR-17: UCI surface unchanged

- GIVEN the pre-feature output of the `uci` handshake captured as a baseline
- WHEN the post-feature engine runs the same handshake
- THEN every `option name ...` line present pre-feature SHALL still be present (possibly alongside new ones)
- AND `id name` and `id author` SHALL still appear
- AND the handshake SHALL still terminate with `uciok`

### Feature-level edge case — KPK safety

- GIVEN a theoretically-won KPK position (e.g. `4k3/8/4K3/4P3/8/8/8/8 w - - 0 1`) at `compute_phase == 0`
- WHEN the engine at the final kept state searches this position to depth 10 with Syzygy enabled
- THEN the tablebase probe SHALL dominate and return a `TABLEBASE_WIN` score
- AND with Syzygy disabled, the engine SHALL still make progress toward promotion (no repetition loop caused by new eval terms mispricing the KP endgame)

---

## 6. Constraints

### In Scope
- Classical search improvements: aspiration windows (FR-01), reverse futility pruning (FR-02), history heuristic (FR-03), check extensions (FR-04), delta pruning in quiescence (FR-05), Late Move Reductions (FR-06)
- Classical evaluation enrichment: mobility (FR-07), king safety with pawn shield (FR-08), passed/isolated/doubled pawns (FR-09), bishop pair (FR-10), rook on open/semi-open file (FR-11)
- Per-patch gauntlet gating via `just measure-elo 20 5 20` with the positive-point-estimate rule (FR-12)
- Baseline re-measurement between patches (FR-13) and clean revert on gate failure (FR-14)
- Run log at `.sddw/push-up-elo/run-log.md` recording baseline and per-patch Elo deltas
- Selective TDD for isolated helpers (history table, pawn-structure detection, mobility counting, bishop-pair count, rook-on-file predicate, delta-pruning predicate, aspiration re-search logic)

### Out of Scope
- NNUE / any neural-network evaluation — explicitly excluded by the user and beyond a one-day budget
- Texel / automated evaluation parameter tuning — requires a labelled dataset and a tuner; revisit in a separate feature
- Singular extensions, multi-cut, ProbCut, internal iterative deepening — larger tuning surface; exceed day budget
- Static Exchange Evaluation (SEE) for capture ordering and qsearch pruning — valuable but larger than the cheap-wins tier; deferred
- Counter-move heuristic, continuation history — depend on history infra but add complexity; deferred
- Pawn hash table — premature until pawn-eval cost is measured; add only if profiling demands it
- Multi-threading, transposition-table, opening-book, Syzygy, or time-management changes — all already working and not the bottleneck
- Changes to UCI option set or wire format — see FR-17

### Prohibitions
- SHALL NOT introduce any neural-network evaluation (FR-15) — the feature is strictly classical
- SHALL NOT weaken, remove, or `#[ignore]` any existing unit/perft/UCI integration test (FR-16) — correctness SHALL NOT be traded for Elo
- SHALL NOT change the UCI protocol surface (FR-17) — existing GUIs and tournament configs remain compatible
- SHALL NOT merge a patch that failed the gauntlet gate (FR-12), regardless of how intuitively correct the change seems — the gate is authoritative
- SHALL NOT bundle multiple unrelated FRs into a single commit — each FR is its own commit so each can be independently reverted on gate failure
- SHALL NOT introduce tuning magic numbers without citing the literature source (e.g. Stockfish, Ethereal, Chess Programming Wiki) or marking them with a `// TODO: tune` comment
- SHALL NOT commit directly to `master`; all patches SHALL land on a dedicated feature branch (e.g. `feat/push-up-elo`) and be merged only after the full day's run completes and the final cumulative Elo has been recorded

### Testing Approach
- Selective TDD — TDD for isolated, deterministic helpers (history table store/lookup, passed-pawn bitboard detection, bishop-pair count, rook-on-open-file predicate, delta-pruning predicate, aspiration re-search loop). Search-flow changes (LMR, RFP, check extension) are validated primarily by the existing test suite plus the gauntlet, because their contract is statistical (Elo delta) rather than per-input deterministic. Every patch SHALL keep `cargo test --all` green (FR-16) before the gauntlet runs.
