# Task 8: Implement opening book and endgame tablebase support

## Trace
- **FR-IDs:** FR-09, FR-10, FR-12, FR-13
- **Depends on:** task-2, task-5

## Files
- `src/book.rs` — create
- `src/tablebase.rs` — create
- `src/search/mod.rs` — modify (integrate book/tablebase probing into search entry)
- `src/uci.rs` — modify (add BookPath/SyzygyPath option handling)
- `Cargo.toml` — modify (add shakmaty + shakmaty-syzygy dependencies)

## Architecture

### Components
- `book` module: Polyglot .bin format reader with dedicated Zobrist hashing — new
- `tablebase` module: Syzygy wrapper using `shakmaty-syzygy` crate with position adapter — new

### Data Flow
```
Search entry:
  1. Check opening book → if hit, return book move (weighted random)
  2. If no book hit, proceed to normal search
  3. During search, at leaf nodes with ≤6 pieces → probe Syzygy tablebase
     → WDL result adjusts score (win=+TABLEBASE_WIN, draw=0, loss=-TABLEBASE_WIN)
     → DTZ result used at root for best move selection among winning moves

Book lookup:
  Position → book::polyglot_hash(pos) → binary search in .bin file → weighted random selection → Move

Tablebase probe:
  Position → adapter → shakmaty::Chess → shakmaty_syzygy::Tablebase::probe_wdl() → WDL score
```

## Contracts

### Internal Interfaces

**book.rs:**
- `PolyglotBook` struct: holds the parsed book entries (sorted by hash)
  - `PolyglotBook::open(path: &Path) -> Result<PolyglotBook, BookError>`: read and parse .bin file
  - `PolyglotBook::probe(&self, pos: &Position) -> Option<Move>`: look up position, return weighted random move
    - Binary search by Polyglot hash key
    - Multiple entries for same key → weighted random selection (weight = entry.weight)
    - Returns `None` if position not in book
  - Pre: book file is valid Polyglot .bin format (SHALL NOT write to file — FR-13)
- `polyglot_hash(pos: &Position) -> u64`: compute Polyglot-specific Zobrist hash
  - Uses Polyglot's own random number table (different from engine's Zobrist keys)
  - Handles castling and en passant encoding per Polyglot spec
- `decode_polyglot_move(raw: u16, pos: &Position) -> Option<Move>`: convert Polyglot move encoding to engine Move
  - Polyglot encodes castling as king-to-rook (e.g., e1h1), must convert to king-to-destination (e1g1)

**Entry format (16 bytes, big-endian):**
| Field  | Type   | Description |
|--------|--------|-------------|
| key    | u64 BE | Polyglot Zobrist hash |
| move   | u16 BE | to_file(3), to_row(3), from_file(3), from_row(3), promotion(3) |
| weight | u16 BE | Selection weight |
| learn  | u32 BE | Ignored |

**tablebase.rs:**
- `SyzygyTablebase` struct: wraps `shakmaty_syzygy::Tablebase`
  - `SyzygyTablebase::new(path: &Path) -> Result<SyzygyTablebase, TablebaseError>`: initialize with path to TB directory
  - `SyzygyTablebase::probe_wdl(&self, pos: &Position) -> Option<WdlResult>`: probe for WDL (Win/Draw/Loss)
    - Returns `None` if position has >6 pieces or tables not available
    - Converts engine `Position` to `shakmaty::Chess` internally
  - `SyzygyTablebase::probe_dtz(&self, pos: &Position) -> Option<DtzResult>`: probe for DTZ (Distance to Zeroing)
    - Used at root for selecting best winning move
- `WdlResult` enum: `Win`, `Draw`, `Loss`, `CursedWin`, `BlessedLoss`
- `DtzResult` struct: `wdl: WdlResult`, `dtz: i32`
- `to_shakmaty(pos: &Position) -> shakmaty::Chess`: adapter function
  - Converts bitboards, piece placement, side to move, castling, en passant

**Search integration:**
- Before iterative deepening: `if book.probe(pos).is_some() → return book move`
- At leaf/low-depth nodes: `if pos.piece_count() <= 6 → probe_wdl() → adjust score`
- At root: `probe_dtz() → prefer moves that win with shortest DTZ`

## Design Decisions

### Polyglot book: from-scratch implementation
- **Chosen:** Implement Polyglot reader from scratch (binary search, big-endian parsing, dedicated Zobrist)
- **Rationale:** Format is simple (sorted array of 16-byte entries). Avoids a dependency. Full control over weighted random selection and move decoding.
- **Rejected:** Using a crate — no well-maintained Rust Polyglot crate exists; the format is simple enough to implement directly

### Syzygy: shakmaty-syzygy crate with adapter
- **Chosen:** Use `shakmaty-syzygy` crate, write an adapter from our `Position` to `shakmaty::Chess`
- **Rationale:** Syzygy probing is complex (compression, DTZ tables). `shakmaty-syzygy` is maintained by niklasf (Lichess), supports up to 7-piece tables, and is the only viable Rust option. The adapter layer is straightforward.
- **Rejected:** Fathom FFI — C dependency complicates builds; from-scratch — disproportionate effort for a feature that's orthogonal to the engine's core

### Book move selection: weighted random
- **Chosen:** Weighted random selection using entry weights
- **Rationale:** Adds variety to opening play while still preferring stronger moves. Standard behavior for Polyglot books.
- **Rejected:** Always play highest-weight move — too predictable; uniform random — ignores quality signals

### Tablebase integration point: search leaf + root
- **Chosen:** WDL probing at leaf nodes (score adjustment), DTZ at root (move selection)
- **Rationale:** WDL at leaves gives the search accurate endgame scores without deep searching. DTZ at root selects the fastest winning line. This is the standard approach.
- **Rejected:** Probing at every node — too many table accesses, diminishing returns

## Acceptance Criteria

### FR-09: Opening book
- GIVEN a Polyglot `.bin` book file is configured
- WHEN the current position has a book entry
- THEN the engine SHALL play a book move (weighted random selection)

- GIVEN no book file is configured or the position is not in the book
- WHEN the engine needs a move
- THEN it SHALL fall back to normal search

### FR-10: Endgame tablebases
- GIVEN Syzygy tablebase files are configured
- WHEN a position has 6 or fewer pieces
- THEN the engine SHALL probe the tablebase for the optimal move

- GIVEN tablebase files are missing or the position has more than 6 pieces
- THEN the engine SHALL fall back to normal search without error

### FR-12: No neural network evaluation
- (Covered by architecture: no NN infrastructure exists)

### FR-13: Read-only external resources
- GIVEN opening book or tablebase files
- WHEN the engine accesses them
- THEN it SHALL NOT modify or write to those files

## Done Criteria
- [ ] `PolyglotBook::open()` correctly reads .bin files (test with a known small book)
- [ ] `polyglot_hash()` produces correct hashes for known positions (verified against reference)
- [ ] Book probe returns moves for positions present in the book
- [ ] Book probe returns `None` for positions not in the book
- [ ] Weighted random selection distributes moves according to weights (statistical test)
- [ ] Polyglot castling encoding (king-to-rook) correctly converted to engine format
- [ ] Syzygy adapter correctly converts Position to shakmaty::Chess
- [ ] Tablebase WDL probe returns correct results for known endgame positions
- [ ] Tablebase probe returns `None` for positions with >6 pieces
- [ ] Missing tablebase files do not cause errors (graceful fallback)
- [ ] Book and tablebase files are opened read-only (no writes)
- [ ] Search integration: book move returned before search when available
- [ ] Search integration: tablebase score used at leaf nodes for ≤6 piece positions
- [ ] UCI options `BookPath` and `SyzygyPath` correctly configure book/tablebase paths
- [ ] Unit tests for Polyglot hash, move decoding, and book lookup
- [ ] Integration test: engine plays book moves in known opening positions
