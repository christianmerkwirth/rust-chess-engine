#!/usr/bin/env python3
# /// script
# dependencies = []
# ///
"""
A wrapper around cutechess-cli to measure the Elo rating of the engine against Stockfish.
Used for quick testing and automated tuning (hill-climbing).
"""

import argparse
import subprocess
import re
import sys
import os

def run_match(args):
    """
    Constructs and runs the cutechess-cli command.
    """
    # Prepare engine and stockfish arguments for cutechess-cli
    engine_args_list = ["-engine", f"cmd={args.engine}"]
    if args.engine_args:
        engine_args_list.append(f"args={args.engine_args}")
    
    sf_args_list = [
        "-engine", f"cmd={args.stockfish}",
        "name=Stockfish",
        f"option.Skill Level={args.sf_skill}"
    ]

    cutechess_args = [
        "cutechess-cli"
    ] + engine_args_list + sf_args_list + [
        "-each", "proto=uci", f"tc={args.tc}",
        "-games", "2",
        "-rounds", str(args.games // 2),
        "-repeat",
        "-concurrency", str(args.concurrency),
        "-ratinginterval", "10",
    ]

    print(f"Running match: {' '.join(cutechess_args)}")
    
    # Run the command and capture output line by line to parse results as they come
    try:
        process = subprocess.Popen(
            cutechess_args,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            bufsize=1
        )
    except FileNotFoundError:
        print("Error: 'cutechess-cli' not found. Please install it to use this script.", file=sys.stderr)
        return None

    elo_diff = None
    error_margin = None
    
    # Pattern to match: Elo difference: -100.2 +/- 20.3, LOS: 0.0 %, DrawRatio: 10.5 %
    # Updated to handle 'nan', 'inf', and '-inf'
    val_re = r"[+-]?\d+\.?\d*|nan|inf|-inf"
    elo_pattern = re.compile(rf"Elo difference:\s+({val_re})\s+/-\s+({val_re})")

    for line in process.stdout:
        print(line, end="")
        match = elo_pattern.search(line)
        if match:
            try:
                e = float(match.group(1))
                # Only overwrite if we got a real number or infinity
                if not math.isnan(e):
                    elo_diff = e
            except ValueError:
                pass
            
            try:
                m = float(match.group(2))
                error_margin = m
            except ValueError:
                pass

    process.wait()
    
    if process.returncode != 0:
        print(f"\nError: cutechess-cli exited with code {process.returncode}", file=sys.stderr)
        # Note: cutechess-cli often exits with non-zero if interrupted or if one engine crashes
        if elo_diff is None:
            return None

    return elo_diff, error_margin

def main():
    parser = argparse.ArgumentParser(description="Measure Elo rating vs Stockfish using cutechess-cli.")
    parser.add_argument("--engine", default="./target/release/chess_engine", help="Path to the engine binary.")
    parser.add_argument("--engine-args", default="", help="Optional arguments for the engine.")
    parser.add_argument("--stockfish", default="stockfish", help="Path to the Stockfish binary.")
    parser.add_argument("--sf-skill", type=int, default=5, help="Stockfish Skill Level (0-20).")
    parser.add_argument("--tc", default="10+0.1", help="Time control (e.g. 10+0.1 or 40/60).")
    parser.add_argument("--games", type=int, default=100, help="Total number of games to play.")
    parser.add_argument("--concurrency", type=int, default=4, help="Number of concurrent games.")
    
    args = parser.parse_args()

    # Ensure engine binary exists
    if not os.path.exists(args.engine):
        print(f"Engine binary not found at {args.engine}.")
        print("Please build it first: cargo build --release")
        sys.exit(1)

    result = run_match(args)
    
    if result:
        elo, margin = result
        print("\n" + "="*40)
        
        import math
        def format_val(v, sign=False):
            if v is None: return "N/A"
            try:
                if math.isnan(v): return "N/A"
                if math.isinf(v): return "+inf" if v > 0 else "-inf"
                return f"{v:+.1f}" if sign else f"{v:.1f}"
            except (TypeError, ValueError):
                return "N/A"
        
        elo_str = format_val(elo, sign=True)
        margin_str = format_val(margin)
        
        print(f"FINAL ESTIMATED ELO DIFF: {elo_str} +/- {margin_str}")
        print("(Positive means chess_engine is stronger than Stockfish Skill {})".format(args.sf_skill))
        print("="*40)
    else:
        print("\nFailed to measure Elo.")
        sys.exit(1)

if __name__ == "__main__":
    main()
