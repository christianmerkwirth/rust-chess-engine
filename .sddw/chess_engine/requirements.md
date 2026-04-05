# Requirements: chess_engine

## 1. Project

- Path: `.`

## 2. Purpose

Build a UCI-compatible chess engine in Rust that provides a competitive, self-contained opponent (~1800 ELO) for use with standard chess GUIs — combining bitboard-based move generation, alpha-beta search with iterative deepening, and classical evaluation to deliver correct and reasonably strong play.

## 3. User Stories

- As a chess player, I want to load the engine in a UCI-compatible GUI (e.g., CuteChess, Arena), so that I can play games against a challenging opponent.
- As a developer, I want to run perft tests against the engine, so that I can verify move generation is correct for all positions.
- As an engine tester, I want to pit this engine against other UCI engines, so that I can measure its relative strength and identify weaknesses.
- As a contributor, I want the engine to be modular (board, search, evaluation, UCI as separate components), so that I can improve individual subsystems independently.

## 4. Functional Requirements

### Core

- FR-01: The engine SHALL represent the board using bitboards (one 64-bit integer per piece type per color).
- FR-02: The engine SHALL generate all legal moves for any valid chess position, including castling, en passant, and pawn promotion.
- FR-03: The engine SHALL implement alpha-beta search with iterative deepening and transposition tables.
- FR-04: The engine SHALL evaluate positions using material values and piece-square tables with separate middlegame and endgame weights.
- FR-05: The engine SHALL implement move ordering (PV move, captures via MVV-LVA, killer moves) to improve search efficiency.

### UCI & Integration

- FR-06: The engine SHALL implement the UCI protocol (`uci`, `isready`, `position`, `go`, `stop`, `quit`, and `ucinewgame` commands at minimum).
- FR-07: The engine SHALL support UCI time controls (`wtime`, `btime`, `winc`, `binc`, `movestogo`, `movetime`, `depth`, `infinite`).
- FR-08: The engine SHALL support pondering (thinking on the opponent's time) via the UCI `go ponder` command.

### Advanced Features

- FR-09: The engine SHALL support opening book lookup (Polyglot `.bin` format).
- FR-10: The engine SHALL support Syzygy endgame tablebase probing for positions with 6 or fewer pieces.
- FR-11: The engine SHALL support multi-threaded search via Lazy SMP.

### Prohibitions

- FR-12: The engine SHALL NOT use neural network evaluation (NNUE or similar).
- FR-13: The engine SHALL NOT modify or write to opening book or tablebase files.

## 5. Acceptance Criteria

### FR-01: Bitboard representation

- GIVEN any standard starting position
- WHEN the board is initialized
- THEN each piece type SHALL have a corresponding 64-bit bitboard with correct bit positions

- GIVEN a FEN string
- WHEN the board is loaded
- THEN the bitboard state SHALL match the FEN exactly

### FR-02: Legal move generation

- GIVEN the standard starting position
- WHEN a perft test is run to depth 6
- THEN the node count SHALL match the known correct value (119,060,324)

- GIVEN a position with castling rights
- WHEN the king or rook has moved or squares are attacked
- THEN castling SHALL only be generated when legal

- GIVEN a position with an en passant opportunity
- WHEN the en passant capture would leave the king in check
- THEN the move SHALL NOT be generated

- GIVEN a position with a pawn on the 7th/2nd rank
- WHEN the pawn advances or captures
- THEN all four promotion types (Q, R, B, N) SHALL be generated

### FR-03: Search

- GIVEN a position with a forced mate in 3
- WHEN the engine searches to sufficient depth
- THEN it SHALL find the mating move

- GIVEN iterative deepening
- WHEN a search completes depth N
- THEN the PV from depth N SHALL be used to order moves at depth N+1

- GIVEN a transposition table entry with sufficient depth
- WHEN the same position is reached again
- THEN the stored evaluation SHALL be reused to avoid redundant search

### FR-04: Evaluation

- GIVEN a position with equal material
- WHEN one side has better piece placement
- THEN the evaluation SHALL favor that side

- GIVEN a position transitioning from middlegame to endgame
- WHEN pieces are traded
- THEN piece-square table weights SHALL interpolate between middlegame and endgame values

### FR-05: Move ordering

- GIVEN a position with the PV move available
- WHEN moves are ordered
- THEN the PV move SHALL be searched first

- GIVEN capturing moves
- WHEN ordered by MVV-LVA
- THEN capturing a queen with a pawn SHALL be ordered before capturing a pawn with a queen

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

### FR-08: Pondering

- GIVEN `bestmove e2e4 ponder e7e5` was sent
- WHEN the GUI sends `go ponder`
- THEN the engine SHALL search the predicted position until `ponderhit` or `stop`

- GIVEN the engine is pondering
- WHEN `ponderhit` is received
- THEN the engine SHALL switch to normal time-managed search

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

### FR-11: Multi-threaded search

- GIVEN the `Threads` UCI option is set to N
- WHEN the engine searches
- THEN it SHALL use N threads with a shared transposition table (Lazy SMP)

- GIVEN multi-threaded search
- WHEN all threads complete
- THEN the best move from the main thread SHALL be reported

### FR-12: No neural network evaluation

- GIVEN any position
- WHEN the engine evaluates
- THEN it SHALL NOT use neural network inference

### FR-13: Read-only external resources

- GIVEN opening book or tablebase files
- WHEN the engine accesses them
- THEN it SHALL NOT modify or write to those files

## 6. Constraints

### In Scope

- Bitboard-based board representation with magic bitboards for sliding pieces
- Full legal move generation (castling, en passant, promotion, check evasion)
- Alpha-beta search with iterative deepening, transposition tables, move ordering (PV, MVV-LVA, killer moves)
- Classical evaluation (material + piece-square tables with middlegame/endgame interpolation)
- Full UCI protocol support (core commands + time management)
- Pondering via UCI `go ponder` / `ponderhit`
- Polyglot opening book support
- Syzygy endgame tablebase probing (6 or fewer pieces)
- Lazy SMP multi-threaded search
- Perft test suite for move generation verification

### Out of Scope

- NNUE or neural network evaluation — classical only for v1
- Custom GUI or TUI — engine is headless, relies on external GUIs
- Online play integration (Lichess/Chess.com API) — engine is a local UCI binary
- Chess960/Fischer Random — standard chess only for v1
- Endgame tablebases with more than 6 pieces — standard Syzygy 6-man limit

### Prohibitions

- SHALL NOT use neural network inference for evaluation
- SHALL NOT write to or modify opening book or tablebase files
- SHALL NOT send UCI output that deviates from the UCI protocol specification
- SHALL NOT panic or crash on malformed UCI input — gracefully ignore invalid commands

### Testing Approach

- TDD — write failing tests first, then implement to pass. Perft tests serve as the primary correctness validation for move generation. Tactical puzzle suites validate search and evaluation strength.
