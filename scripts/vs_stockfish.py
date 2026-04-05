#!/usr/bin/env python3
"""
Play N games between chess_engine and Stockfish, then print results.

Usage:
    python3 scripts/vs_stockfish.py [--games N] [--skill LEVEL] [--movetime MS]
"""

import argparse
import subprocess
import threading
import time
import chess
import chess.pgn
import sys
from dataclasses import dataclass, field
from typing import Optional


# ── UCI engine wrapper ──────────────────────────────────────────────────────

class UCIEngine:
    def __init__(self, path: str, name: str):
        self.name = name
        self.proc = subprocess.Popen(
            [path],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            text=True,
            bufsize=1,
        )
        self._lock = threading.Lock()

    def send(self, cmd: str):
        self.proc.stdin.write(cmd + "\n")
        self.proc.stdin.flush()

    def read_until(self, keyword: str, timeout: float = 10.0) -> list[str]:
        lines = []
        deadline = time.time() + timeout
        while time.time() < deadline:
            line = self.proc.stdout.readline().rstrip("\n")
            if line:
                lines.append(line)
            if line.startswith(keyword):
                return lines
        raise TimeoutError(f"{self.name}: timed out waiting for '{keyword}'")

    def uci_init(self):
        self.send("uci")
        self.read_until("uciok")
        self.send("isready")
        self.read_until("readyok")

    def set_option(self, name: str, value):
        self.send(f"setoption name {name} value {value}")

    def new_game(self):
        self.send("ucinewgame")
        self.send("isready")
        self.read_until("readyok")

    def best_move(self, fen: str, moves: list[str], movetime_ms: int) -> Optional[str]:
        move_str = " ".join(moves) if moves else ""
        if move_str:
            self.send(f"position fen {fen} moves {move_str}")
        else:
            self.send(f"position fen {fen}")
        self.send(f"go movetime {movetime_ms}")
        lines = self.read_until("bestmove", timeout=movetime_ms / 1000 + 5)
        for line in reversed(lines):
            if line.startswith("bestmove"):
                parts = line.split()
                mv = parts[1] if len(parts) > 1 else None
                return None if mv in (None, "(none)") else mv
        return None

    def quit(self):
        try:
            self.send("quit")
            self.proc.wait(timeout=2)
        except Exception:
            self.proc.kill()


# ── Game logic ───────────────────────────────────────────────────────────────

@dataclass
class GameResult:
    winner: Optional[str]   # engine name, or None for draw
    reason: str
    moves: int
    pgn: str


def play_game(
    white: UCIEngine,
    black: UCIEngine,
    movetime_ms: int,
    game_num: int,
) -> GameResult:
    board = chess.Board()
    white.new_game()
    black.new_game()

    move_history: list[str] = []
    start_fen = chess.STARTING_FEN

    pgn_game = chess.pgn.Game()
    pgn_game.headers["White"] = white.name
    pgn_game.headers["Black"] = black.name
    pgn_game.headers["Round"] = str(game_num)
    node = pgn_game

    while not board.is_game_over(claim_draw=True):
        engine = white if board.turn == chess.WHITE else black

        mv_uci = engine.best_move(start_fen, move_history, movetime_ms)
        if mv_uci is None:
            # Engine returned no move — treat as resign
            winner = black.name if board.turn == chess.WHITE else white.name
            pgn_game.headers["Result"] = "1-0" if winner == white.name else "0-1"
            return GameResult(winner, "resign/no-move", len(move_history), str(pgn_game))

        try:
            move = board.parse_uci(mv_uci)
        except ValueError:
            winner = black.name if board.turn == chess.WHITE else white.name
            pgn_game.headers["Result"] = "1-0" if winner == white.name else "0-1"
            return GameResult(winner, f"illegal move {mv_uci}", len(move_history), str(pgn_game))

        node = node.add_variation(move)
        board.push(move)
        move_history.append(mv_uci)

    outcome = board.outcome(claim_draw=True)
    if outcome is None or outcome.winner is None:
        pgn_game.headers["Result"] = "1/2-1/2"
        return GameResult(None, board.result(claim_draw=True), len(move_history), str(pgn_game))

    winner = white.name if outcome.winner == chess.WHITE else black.name
    pgn_game.headers["Result"] = "1-0" if outcome.winner == chess.WHITE else "0-1"
    reason = outcome.termination.name.lower()
    return GameResult(winner, reason, len(move_history), str(pgn_game))


# ── Main ─────────────────────────────────────────────────────────────────────

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--games",    type=int, default=7,    help="Number of games")
    ap.add_argument("--skill",    type=int, default=3,    help="Stockfish Skill Level (0-20)")
    ap.add_argument("--movetime", type=int, default=100,  help="Milliseconds per move")
    ap.add_argument("--engine",   type=str, default="./target/release/chess_engine")
    ap.add_argument("--stockfish",type=str, default="/usr/games/stockfish")
    args = ap.parse_args()

    print(f"Match: chess_engine vs Stockfish (Skill {args.skill})")
    print(f"Games: {args.games}  |  Move time: {args.movetime}ms/move")
    print("=" * 55)

    results = []
    engine_wins = draws = stockfish_wins = 0

    for g in range(1, args.games + 1):
        # Alternate colours every game
        if g % 2 == 1:
            white_path, black_path = args.engine, args.stockfish
            white_name, black_name = "chess_engine", f"Stockfish(S{args.skill})"
        else:
            white_path, black_path = args.stockfish, args.engine
            white_name, black_name = f"Stockfish(S{args.skill})", "chess_engine"

        white = UCIEngine(white_path, white_name)
        black = UCIEngine(black_path, black_name)

        white.uci_init()
        black.uci_init()

        if white_name.startswith("Stockfish"):
            white.set_option("Skill Level", args.skill)
            white.set_option("UCI_LimitStrength", "true")
        if black_name.startswith("Stockfish"):
            black.set_option("Skill Level", args.skill)
            black.set_option("UCI_LimitStrength", "true")

        print(f"Game {g:2d}: {white_name:30s} (W) vs {black_name:30s} (B) ... ", end="", flush=True)

        try:
            result = play_game(white, black, args.movetime, g)
        except Exception as e:
            print(f"ERROR: {e}")
            white.quit(); black.quit()
            continue

        white.quit()
        black.quit()
        results.append(result)

        if result.winner is None:
            draws += 1
            tag = "½-½"
        elif result.winner == "chess_engine":
            engine_wins += 1
            tag = "chess_engine wins"
        else:
            stockfish_wins += 1
            tag = "Stockfish wins"

        print(f"{tag:22s}  ({result.moves} moves, {result.reason})")

    # ── Summary ──────────────────────────────────────────────────────────────
    total = engine_wins + draws + stockfish_wins
    print()
    print("=" * 55)
    print(f"Results after {total} game(s):")
    print(f"  chess_engine : {engine_wins} wins")
    print(f"  Draws        : {draws}")
    print(f"  Stockfish    : {stockfish_wins} wins")
    if total:
        score = engine_wins + 0.5 * draws
        print(f"  Score        : {score}/{total}  ({100*score/total:.1f}%)")
    print("=" * 55)


if __name__ == "__main__":
    main()
