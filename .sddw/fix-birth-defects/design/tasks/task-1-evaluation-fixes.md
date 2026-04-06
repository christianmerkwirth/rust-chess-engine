# Task 1: Evaluation Fixes

Fix the critical evaluation bugs related to PST indexing, the middlegame king table, duplicated PST rows, and non-tapered material values.

## 1. Requirements

- FR-01: Correct PST indexing (white prefers pushing pawns).
- FR-02: Correct middlegame king PST (prefers back rank/castling over center).
- FR-03: Replace duplicated Bishop MG and Queen MG row 7 with correct PeSTO values.
- FR-04: Use distinct tapered MG/EG material values (PeSTO values).
- FR-19: Add eval-sanity regression tests.

## 2. Design

### PST Indexing (FR-01)
- The current implementation in `src/eval/mod.rs` flips the square for black but not for white.
- The PST tables in `src/eval/pst.rs` are in PeSTO visual/BERF layout (row 0 = rank 8).
- For LERF squares (`a1=0`), white must flip to match this layout, and black must not.
- Change `src/eval/mod.rs:28-32`:
  ```rust
  let eval_sq = if color == Color::White {
      (sq.0 ^ 56) as usize // Flip for white to match BERF table
  } else {
      sq.0 as usize        // No flip for black
  };
  ```

### Middlegame King Table (FR-02)
- Replace `PST_MG[King]` in `src/eval/pst.rs` with values that penalize the king moving to the center in the middlegame.
- PeSTO King MG (BERF):
  ```
  -24, -34, -34, -34, -34, -34, -34, -24,
  -24, -34, -34, -34, -34, -34, -34, -24,
  -24, -34, -34, -34, -34, -34, -34, -24,
  -24, -34, -34, -34, -34, -34, -34, -24,
  -24, -34, -34, -34, -34, -34, -34, -24,
  -24, -34, -34, -34, -34, -34, -34, -24,
  -20, -20, -20, -20, -20, -20, -20, -20,
    0,  20,  40, -20,   0, -20,  40,  20,
  ```

### Duplicated and Corrected PST Rows (FR-03)
- Update `PST_MG[Bishop]` and `PST_MG[Queen]` in `src/eval/pst.rs` to match published PeSTO values, ensuring row 7 is not a duplicate.
- Audit all PST tables against PeSTO to ensure they match the BERF layout expected by the evaluator.

### Tapered Material Values (FR-04)
- Update `MATERIAL_MG` and `MATERIAL_EG` in `src/eval/mod.rs` (or `pst.rs` if moved) to use PeSTO values:
  - `MG = [82, 337, 365, 477, 1025, 0]`
  - `EG = [94, 281, 297, 512, 936, 0]`

### Eval Sanity Tests (FR-19)
- Add tests to `src/eval/mod.rs` (in `mod tests`):
  - `eval_e2e4_improves_white`: `evaluate(startpos + e2e4) > evaluate(startpos)`.
  - `eval_a7_beats_a2`: `evaluate(pos_with_wP_a7) > evaluate(pos_with_wP_a2)`.
  - `king_mg_prefers_back_rank`: `evaluate(pos_K_e1) > evaluate(pos_K_e4)` and `evaluate(pos_K_g1) > evaluate(pos_K_e4)` in a middlegame phase.
  - `pst_bishop_row7_matches_pesto`: Assert `PST_MG[Bishop]` row 7 matches known good value.
  - `material_mg_ne_eg`: Assert `MATERIAL_MG != MATERIAL_EG`.

## 3. Files to Modify

- `src/eval/mod.rs`: Update indexing logic, material constants, and add tests.
- `src/eval/pst.rs`: Update PST table values.

## 4. Verification

- `cargo test src/eval/mod.rs` (to run the new sanity tests).
- Manual inspection of `evaluate` outputs for specific positions.
