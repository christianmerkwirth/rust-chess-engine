# Task 4: Implement evaluation with material and piece-square tables

## Trace
- **FR-IDs:** FR-04
- **Depends on:** task-1, task-2

## Files
- `src/eval/mod.rs` — create
- `src/eval/pst.rs` — create
- `src/lib.rs` — modify (add `eval` module)

## Architecture

### Components
- `eval` module: position evaluation entry point with tapered eval — new
- `pst` module: piece-square tables with separate middlegame and endgame values — new

### Data Flow
Position → `eval::evaluate()` → compute game phase → sum material + PST for mg and eg → interpolate via tapered eval → return centipawn score from side-to-move perspective

## Contracts

### Internal Interfaces

**eval/mod.rs:**
- `evaluate(pos: &Position) -> i32`: evaluate position in centipawns from side-to-move perspective
  - Pre: position is valid
  - Post: positive = side to move is better, negative = opponent is better
  - Returns 0 for drawn positions (material only)
- `compute_phase(pos: &Position) -> i32`: game phase score 0 (endgame) to 24 (opening)
  - Phase values: P=0, N=1, B=1, R=2, Q=4, total max=24

**eval/pst.rs:**
- `MATERIAL_MG[Piece]` and `MATERIAL_EG[Piece]`: material values for middlegame and endgame
  - MG: P=100, N=320, B=330, R=500, Q=900, K=0
  - EG: P=100, N=320, B=330, R=500, Q=900, K=0 (can be tuned separately later)
- `PST_MG[Piece][Square]`: middlegame piece-square table values (centipawns, from White's perspective)
- `PST_EG[Piece][Square]`: endgame piece-square table values
- Tables defined for White; mirror vertically for Black
- Values sourced from well-known PST values (PeSTO or similar)

**Tapered eval formula:**
```
mg_score = sum of (material_mg[piece] + pst_mg[piece][sq]) for all white pieces
         - sum of (material_mg[piece] + pst_mg[piece][mirror(sq)]) for all black pieces
eg_score = (same with _eg tables)
phase = compute_phase(pos)  // 0..24
score = (mg_score * phase + eg_score * (24 - phase)) / 24
return score * (1 if white to move else -1)  // relative to side to move
```

## Design Decisions

### Evaluation interpolation: tapered eval
- **Chosen:** Tapered eval with phase score based on remaining non-pawn material
- **Rationale:** Smooth transition between middlegame and endgame evaluation. Standard approach (Stockfish, Fruit, CPW). Phase = sum of piece phase values (P=0, N=1, B=1, R=2, Q=4, max=24).
- **Rejected:** Discrete phase detection (opening/middlegame/endgame) — creates discontinuities at boundaries, less accurate

### PST values: PeSTO-based
- **Chosen:** Use well-known PeSTO piece-square table values as starting point
- **Rationale:** PeSTO tables are well-tuned, publicly available, and known to produce ~1800 ELO with basic search. Good baseline that can be tuned later.
- **Rejected:** Custom-tuned from scratch — requires automated tuning infrastructure not in scope

### Score perspective: side-to-move relative
- **Chosen:** `evaluate()` returns score relative to side to move (positive = good for side to move)
- **Rationale:** Simplifies negamax search — just negate the score at each level. Standard convention.
- **Rejected:** Always-white perspective — requires sign flipping in search, error-prone

## Acceptance Criteria

### FR-04: Evaluation
- GIVEN a position with equal material
- WHEN one side has better piece placement
- THEN the evaluation SHALL favor that side

- GIVEN a position transitioning from middlegame to endgame
- WHEN pieces are traded
- THEN piece-square table weights SHALL interpolate between middlegame and endgame values

## Done Criteria
- [ ] `evaluate()` returns correct material balance for positions with unequal material
- [ ] PST values favor central knight positions over rim positions in middlegame
- [ ] PST values favor king safety (castled position) in middlegame, king centralization in endgame
- [ ] Tapered eval smoothly interpolates: opening position uses mostly MG weights, king+pawn endgame uses mostly EG weights
- [ ] `compute_phase()` returns 24 for starting position, 0 for king+pawns only
- [ ] Score is relative to side to move (symmetric: eval(pos) from white == -eval(pos) from black)
- [ ] Starting position evaluates to approximately 0 (small advantage for white due to tempo is acceptable)
- [ ] Unit tests verify material counting, PST scoring, and phase interpolation independently
