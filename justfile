# Chess Engine — task runner
# Run `just` or `just help` to list all commands.

export PATH := env_var('HOME') + "/.cargo/bin:" + env_var('PATH')

default: help

# List all available commands
help:
    @just --list

# Build the engine (debug)
build:
    cargo build

# Build the engine (optimised release binary)
release:
    cargo build --release

# Run the engine UCI loop (release build)
run: release
    ./target/release/chess_engine

# Run all unit tests (library crate)
test:
    cargo test --lib

# Run all tests including integration tests (UCI + perft)
test-all:
    cargo test

# Run only the UCI integration tests
test-uci:
    cargo test --test uci_tests

# Run only the perft correctness tests
test-perft:
    cargo test --test perft_tests

# Run a quick engine self-play search from the starting position (depth 8)
bench:
    @echo "Searching starting position at depth 8..."
    @printf 'uci\nisready\nposition startpos\ngo depth 8\n' | ./target/release/chess_engine

# Benchmark NPS across thread counts (1, 2, 4)
bench-smp: release
    @echo "=== 1 thread ==="
    @printf 'uci\nisready\nsetoption name Threads value 1\nposition fen r2q1rk1/pp2bppp/2n1bn2/3pp3/2B1P3/2NP1N1P/PPP2PP1/R1BQ1RK1 w - - 0 1\ngo depth 9\n' \
        | ./target/release/chess_engine | grep 'depth 9\|bestmove'
    @echo ""
    @echo "=== 2 threads ==="
    @printf 'uci\nisready\nsetoption name Threads value 2\nposition fen r2q1rk1/pp2bppp/2n1bn2/3pp3/2B1P3/2NP1N1P/PPP2PP1/R1BQ1RK1 w - - 0 1\ngo depth 9\n' \
        | ./target/release/chess_engine | grep 'depth 9\|bestmove'
    @echo ""
    @echo "=== 4 threads ==="
    @printf 'uci\nisready\nsetoption name Threads value 4\nposition fen r2q1rk1/pp2bppp/2n1bn2/3pp3/2B1P3/2NP1N1P/PPP2PP1/R1BQ1RK1 w - - 0 1\ngo depth 9\n' \
        | ./target/release/chess_engine | grep 'depth 9\|bestmove'

# Solve a mate-in-2 puzzle (should find the move quickly)
puzzle: release
    @echo "Puzzle: White to move — find the winning continuation"
    @printf 'uci\nisready\nposition fen r1bk3r/p2pBpNp/n4n2/1p1NP2P/6P1/3P4/P1P1K3/q5b1 w - - 1 0\ngo depth 5\n' \
        | ./target/release/chess_engine | grep 'bestmove'

# Play games vs Stockfish: `just vs-stockfish [games] [skill]`  e.g. `just vs-stockfish 10 5`
vs-stockfish games="7" skill="3": release
    python3 scripts/vs_stockfish.py --games {{games}} --skill {{skill}}

# Remove build artefacts
clean:
    cargo clean

# Check code compiles without warnings
check:
    cargo check

# Run clippy lints
lint:
    cargo clippy -- -D warnings

# Format source code
fmt:
    cargo fmt

# Format check (CI-friendly, no writes)
fmt-check:
    cargo fmt -- --check
