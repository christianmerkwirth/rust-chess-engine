# Task 12: Rook on Open/Semi-open File (FR-11)

Add bonuses for rooks on open and semi-open files.

## 1. Requirements

- FR-11: Bonus for rook on open file (no pawns of either colour) and semi-open file (no friendly pawns, but enemy pawns exist).

## 2. Design

### Rook on File Logic
- In `src/eval/mod.rs`, either in the rook piece loop or in a post-loop pass.
- For each rook square:
  - Determine file: `file_bb = FILE_A << (sq % 8)`.
  - Open file: `(white_pawns | black_pawns) & file_bb == 0`.
  - Semi-open file: `our_pawns & file_bb == 0 && enemy_pawns & file_bb != 0`.
- Add `ROOK_OPEN_MG/EG` or `ROOK_SEMI_OPEN_MG/EG` to `mg[side]` and `eg[side]`.

## 3. Files to Modify

- `src/eval/mod.rs`

## 4. Verification

- Symmetry test in `mod.rs`.
- `cargo test --all`
- Run gauntlet `just measure-elo 20 5 20`.
- Keep if `post_elo > baseline`, else revert.
