# Rust Chess Engine

A high-performance, UCI-compliant chess engine written in Rust (~1800 ELO).

## Features

### Core & Representation
- **Bitboard Engine:** Optimized 64-bit board representation for all piece types.
- **Magic Bitboards:** High-speed sliding piece attack generation (Bishops, Rooks, Queens).
- **Legal Move Generation:** Fully compliant move generator including castling, en passant, and promotions.

### Search & Evaluation
- **Alpha-Beta Search:** Negamax formulation with fail-soft pruning and quiescence search.
- **Iterative Deepening:** Time-managed search that progressively explores deeper plies.
- **Transposition Table:** Lock-free implementation using the Hyatt/Mann XOR verification trick for efficient multi-threading.
- **Move Ordering:** Principal Variation (PV), MVV-LVA (Most Valuable Victim - Least Valuable Attacker), and Killer Move heuristics.
- **Tapered Evaluation:** Classical evaluation with material values and PeSTO-based Piece-Square Tables (PST) that interpolate between middlegame and endgame.

### Advanced Capabilities
- **UCI Protocol:** Full support for `uci`, `isready`, `position`, `go`, `stop`, `quit`, and `ucinewgame`.
- **Time Management:** Smart allocation for `wtime`, `btime`, `winc`, `binc`, `movestogo`, and `movetime`.
- **Pondering:** Support for thinking on the opponent's time (`go ponder`).
- **Lazy SMP:** Efficient multi-threaded search using a shared hash table.
- **Opening Books:** Support for Polyglot `.bin` files with weighted random move selection.
- **Endgame Tablebases:** Integration with Syzygy tablebases (up to 6 pieces) via `shakmaty-syzygy`.

## Universal Chess Interface (UCI)

This engine implements the **[Universal Chess Interface (UCI)](https://www.wbec-ridderkerk.nl/html/UCIProtocol.html)**, an open communication protocol that allows chess engines to communicate with graphical user interfaces (GUIs).

By using UCI, you can load this engine into any compatible GUI (like [CuteChess](https://cutechess.com/), [Arena](http://www.playwitharena.de/), or [Bankiagui](https://github.com/upv-clp/bankiagui)) to play games, analyze positions, or run engine-vs-engine tournaments.

## How to Play

To play against this engine, you can use any existing **UCI-compatible Chess GUI**. Since the engine is a headless binary that speaks the Universal Chess Interface (UCI) protocol, you simply "load" it into a GUI.

### 1. Build the Engine
First, ensure you have a compiled release binary:
```bash
just release
# Or manually: cargo build --release
```
The binary will be located at: `./target/release/chess_engine`

### 2. Download a Chess GUI
If you don't have one, here are the most popular choices:
*   **[CuteChess](https://cutechess.com/) (Recommended):** Modern, cross-platform, and very stable.
*   **[Arena Chess GUI](http://www.playwitharena.de/):** Feature-rich and classic (Windows/Linux).

### 3. Load the Engine into the GUI
While every GUI is slightly different, the steps are generally:
1.  **Open Settings/Engines:** Look for a menu like "Engines" -> "Configure" or "Add New".
2.  **Add New Engine:**
    *   **Name:** `Rust Chess Engine` (or anything you like).
    *   **Protocol:** Select **UCI**.
    *   **Path:** Browse to your project folder and select the `./target/release/chess_engine` file.
3.  **Start a Game:** Create a new game and select the "Rust Chess Engine" as your opponent.

### 4. Run Automated Tournaments (CLI)
For automated testing, benchmarking, or engine-vs-engine tournaments, you can use the **[cutechess-cli](https://github.com/cutechess/cutechess)**. This is the standard tool for running headless matches.

Example command to run a 20-game match against Stockfish:
```bash
cutechess-cli \
  -engine cmd=./target/release/chess_engine name=chess_engine \
  -engine cmd=/usr/games/stockfish name=Stockfish \
  -each proto=uci tc=10+0.1 \
  -games 20 \
  -repeat \
  -concurrency 4 \
  -pgnout tournament_results.pgn
```

*   **`tc=10+0.1`**: 10 seconds per game + 0.1s increment.
*   **`-repeat`**: Each engine plays both sides of every opening.
*   **`-concurrency 4`**: Run 4 games in parallel.
*   **`-pgnout`**: Save all game records to a PGN file.

#### Adjusting Opponent Strength
If Stockfish is too dominant, you can dial down its strength using its built-in UCI options:

*   **Skill Level (0-20)**: `0` is weakest, `20` is full strength.
    ```bash
    -engine cmd=/usr/games/stockfish name=Stockfish option."Skill Level"=5
    ```
*   **Elo Limiting (1320-3190)**: Use `UCI_LimitStrength` alongside `UCI_Elo`.
    ```bash
    -engine cmd=/usr/games/stockfish name=Stockfish option.UCI_LimitStrength=true option.UCI_Elo=1500
    ```

## Development & Task Running

The project uses `just` as a command runner. You can run `just` or `just help` to see all available recipes.

| Command | Description |
|---------|-------------|
| `just build` | Build the engine in debug mode |
| `just release` | Build the optimised release binary |
| `just run` | Build and run the engine's UCI loop |
| `just test` | Run all unit tests |
| `just test-all` | Run all tests (unit + integration + perft) |
| `just test-uci` | Run UCI integration tests specifically |
| `just test-perft` | Run perft correctness tests |
| `just bench` | Search the starting position at depth 8 |
| `just bench-smp` | Benchmark NPS across thread counts (1, 2, 4) |
| `just puzzle` | Solve a sample mate-in-2 puzzle |
| `just vs-stockfish` | Play test games against Stockfish (requires `scripts/vs_stockfish.py`) |
| `just lint` / `just fmt` | Run clippy lints and format code |

## Development Process

This project followed a Spec-Driven Development Workflow (SDDW) using the `.sddw/` directory to maintain a single source of truth for requirements and design.

- **Requirements & Design:** Orchestrated by **Claude Code** using **Opus 4.6**.
- **Implementation:** Alternatingly executed by **Gemini CLI** (using **Model Auto Setting 3.1 Pro + 3.0 Flash**) and **Claude Code** (on **Sonnet 4.1**) to optimize daily token usage.
- **Efficiency:** The entire implementation phase, consisting of 9 structured tasks, took approximately **6 hours** of wall clock time (25-40 minutes per task).
- **Interactivity:** Most user interaction occurred during the initial requirements and design phases; the implementation was largely autonomous, requiring very few questions to be answered during the coding tasks.

## Getting Started

### Prerequisites
- [Rust](https://www.rust-lang.org/) (2021 edition)

### Building
```bash
cargo build --release
```

### Running
The engine communicates via the UCI protocol. You can run it directly in the terminal to issue commands or load the binary into any standard chess GUI (e.g., Arena, CuteChess, Stockfish).

```bash
./target/release/chess_engine
```

### Testing
- **Unit Tests:** `cargo test`
- **Perft Tests:** `cargo test --test perft_tests`
- **UCI Integration Tests:** `cargo test --test uci_tests`

## License
[Insert License Here - e.g., MIT or GPLv3]
