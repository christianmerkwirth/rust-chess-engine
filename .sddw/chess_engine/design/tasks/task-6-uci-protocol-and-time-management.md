# Task 6: Implement UCI protocol and time management

## Trace
- **FR-IDs:** FR-06, FR-07
- **Depends on:** task-2, task-5

## Files
- `src/uci.rs` — create
- `src/time.rs` — create
- `src/main.rs` — create

## Architecture

### Components
- `uci` module: UCI protocol parser, command dispatch, response formatting — new
- `time` module: time allocation strategy for different time controls — new
- `main.rs`: entry point, initializes engine and starts UCI loop — new

### Data Flow
```
stdin → uci::parse_command() → UciCommand enum
  → "uci" → respond with id + options + uciok
  → "isready" → ensure init complete → readyok
  → "position" → build Position from startpos/fen + moves
  → "go" → time::allocate_time(limits) → spawn search thread →
            search::iterative_deepening(pos, limits, |info| uci::send_info(info))
            → bestmove
  → "stop" → set stop flag → search returns → bestmove
  → "ucinewgame" → clear TT, reset state
  → "quit" → exit process
stdout ← uci::send() — all output goes through centralized send function
```

## Contracts

### Internal Interfaces

**uci.rs:**
- `UciCommand` enum: `Uci`, `IsReady`, `Position { fen: Option<String>, moves: Vec<String> }`, `Go(GoParams)`, `Stop`, `Quit`, `UciNewGame`, `SetOption { name: String, value: String }`, `Unknown`
- `GoParams` struct: `wtime: Option<u64>`, `btime: Option<u64>`, `winc: Option<u64>`, `binc: Option<u64>`, `movestogo: Option<u32>`, `depth: Option<u32>`, `nodes: Option<u64>`, `movetime: Option<u64>`, `infinite: bool`, `ponder: bool`
- `parse_command(line: &str) -> UciCommand`: parse a UCI input line
  - Post: returns `Unknown` for malformed input (SHALL NOT panic — FR prohibition)
- `uci_loop(engine: &mut Engine)`: main stdin loop, reads lines, dispatches commands
  - Runs search on a separate thread so `stop` can be received during search
- `send_info(info: &SearchInfo)`: format and print `info depth ... score ... nodes ... nps ... pv ...`
- `send_bestmove(mv: Move, ponder: Option<Move>)`: print `bestmove e2e4 [ponder e7e5]`
- `send_id()`: print `id name ChessEngine` and `id author [author]`
- `send_options()`: print UCI options (Hash size, Threads, OwnBook, SyzygyPath)

**Engine struct (in uci.rs or a dedicated engine.rs):**
- `Engine` struct: holds `Position`, `TranspositionTable`, configuration, search thread handle
- `Engine::new() -> Engine`: initialize with default settings
- `Engine::set_option(&mut self, name: &str, value: &str)`: handle `setoption`
- UCI options to support:
  - `Hash` (spin, default 64, min 1, max 4096): TT size in MB
  - `Threads` (spin, default 1, min 1, max 128): number of search threads
  - `OwnBook` (check, default true): use opening book
  - `BookPath` (string): path to Polyglot .bin file
  - `SyzygyPath` (string): path to Syzygy tablebase directory

**time.rs:**
- `TimeControl` struct: parsed time control parameters for the side to move
- `allocate_time(params: &GoParams, side: Color) -> SearchLimits`
  - `movetime` → use that exact time (minus small safety margin)
  - `infinite` → no time limit, search until `stop`
  - `depth` → depth limit only
  - `wtime/btime` with `movestogo` → divide remaining time by moves to go + buffer
  - `wtime/btime` without `movestogo` → estimate moves remaining (~30), use `time/moves_left + increment * 0.75`
  - Safety margin: always reserve ~50ms to avoid flagging
  - Post: returns `SearchLimits` with allocated time and/or depth limit

**Move string conversion (for UCI I/O):**
- `Move::to_uci(&self) -> String`: e.g., "e2e4", "e7e8q" (with promotion letter)
- `Move::from_uci(s: &str, pos: &Position) -> Option<Move>`: parse UCI move string, look up in legal moves to resolve flags

## Design Decisions

### UCI parsing: hand-written parser
- **Chosen:** Simple `split_whitespace()` + match-based parsing, no parser library
- **Rationale:** UCI protocol is simple and line-based. A few match arms handle all commands. No need for a parsing library.
- **Rejected:** `nom` or regex-based parsing — overkill for a simple text protocol

### Search thread: single dedicated thread
- **Chosen:** Run search on a spawned thread, main thread continues reading stdin for `stop`/`quit`
- **Rationale:** UCI requires the engine to accept `stop` while searching. A dedicated search thread with an `AtomicBool` stop flag is the standard solution.
- **Rejected:** Async/tokio — unnecessary complexity for blocking I/O

### Time management: simple fraction with safety margin
- **Chosen:** Allocate `remaining_time / estimated_moves_left + fraction_of_increment`, with a 50ms safety margin
- **Rationale:** Simple, effective at this level. The engine can soft-stop (finish current iteration) or hard-stop (abort mid-search) based on time allocation. Avoids complex time management that requires tuning.
- **Rejected:** Complex adaptive time management — requires extensive testing and tuning, diminishing returns at ~1800 ELO

## Acceptance Criteria

### FR-06: UCI protocol
- GIVEN the engine is started
- WHEN `uci` is sent
- THEN the engine SHALL respond with `id name`, `id author`, and `uciok`

- GIVEN the engine is idle
- WHEN `isready` is sent
- THEN the engine SHALL respond with `readyok`

- GIVEN a `position startpos moves e2e4 e7e5`
- WHEN `go depth 10` is sent
- THEN the engine SHALL output `info` lines and a `bestmove` in UCI format

- GIVEN the engine is searching
- WHEN `stop` is sent
- THEN the engine SHALL halt search and immediately return `bestmove`

### FR-07: Time controls
- GIVEN `go wtime 60000 btime 60000 winc 1000 binc 1000`
- WHEN the engine searches
- THEN it SHALL allocate time proportionally and not exceed its remaining time

- GIVEN `go movetime 5000`
- WHEN the engine searches
- THEN it SHALL return a move within approximately 5 seconds

- GIVEN `go infinite`
- WHEN the engine searches
- THEN it SHALL search until `stop` is received

## Done Criteria
- [ ] `parse_command()` correctly parses all UCI commands listed above
- [ ] Malformed UCI input returns `Unknown`, does not panic or crash
- [ ] `uci` command responds with id, options, and `uciok`
- [ ] `isready` responds with `readyok`
- [ ] `position startpos` sets up the starting position correctly
- [ ] `position fen <fen> moves <moves>` applies moves from a FEN correctly
- [ ] `go depth N` searches to exactly depth N and returns bestmove
- [ ] `go movetime M` returns a move within M milliseconds (±100ms tolerance)
- [ ] `go wtime/btime/winc/binc` allocates time proportionally, doesn't flag
- [ ] `go infinite` searches until `stop` is received
- [ ] `stop` during search returns bestmove within ~100ms
- [ ] `ucinewgame` clears TT
- [ ] `setoption name Hash value N` resizes TT
- [ ] `info` lines include depth, score (cp or mate), nodes, nps, pv
- [ ] `bestmove` format is correct (e.g., `bestmove e2e4`)
- [ ] Move strings convert correctly (including promotions like `e7e8q`)
- [ ] Engine runs as a standalone binary and interacts correctly with a UCI GUI
- [ ] Integration tests verify command/response sequences
