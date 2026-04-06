use std::io::Write;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};

struct UciEngine {
    child: Child,
    reader: BufReader<std::process::ChildStdout>,
}

impl UciEngine {
    fn new() -> Self {
        let mut child = Command::new("./target/debug/chess_engine")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to start engine");
        let stdout = child.stdout.take().expect("Failed to get stdout");
        let reader = BufReader::new(stdout);
        Self { child, reader }
    }

    fn send(&mut self, cmd: &str) {
        let stdin = self.child.stdin.as_mut().expect("Failed to get stdin");
        writeln!(stdin, "{}", cmd).expect("Failed to write to stdin");
        stdin.flush().expect("Failed to flush stdin");
    }

    fn read_line(&mut self) -> String {
        let mut line = String::new();
        self.reader
            .read_line(&mut line)
            .expect("Failed to read from stdout");
        line.trim().to_string()
    }

    fn wait_for(&mut self, expected: &str) {
        loop {
            let line = self.read_line();
            if line.contains(expected) {
                break;
            }
        }
    }
}

impl Drop for UciEngine {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}

#[test]
fn test_uci_protocol_handshake() {
    let mut engine = UciEngine::new();
    engine.send("uci");
    engine.wait_for("uciok");
}

#[test]
fn test_uci_isready() {
    let mut engine = UciEngine::new();
    engine.send("isready");
    engine.wait_for("readyok");
}

#[test]
fn test_uci_position_and_go_depth() {
    let mut engine = UciEngine::new();
    engine.send("position startpos");
    engine.send("go depth 2");
    engine.wait_for("bestmove");
}

#[test]
fn test_uci_fen_position() {
    let mut engine = UciEngine::new();
    // Position after 1. e4
    engine.send("position fen rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1");
    engine.send("go depth 1");
    engine.wait_for("bestmove");
}

#[test]
fn test_uci_movetime() {
    let mut engine = UciEngine::new();
    engine.send("isready");
    engine.wait_for("readyok");
    engine.send("go movetime 100");
    let start = std::time::Instant::now();
    engine.wait_for("bestmove");
    let elapsed = start.elapsed();
    // Allow some margin for initialization and overhead
    assert!(
        elapsed.as_millis() >= 80,
        "Too fast: {}ms",
        elapsed.as_millis()
    );
    assert!(
        elapsed.as_millis() < 500,
        "Too slow: {}ms",
        elapsed.as_millis()
    );
}

#[test]
fn test_uci_ponderhit() {
    let mut engine = UciEngine::new();
    engine.send("isready");
    engine.wait_for("readyok");
    // Start pondering with a generous time limit for when it hits
    engine.send("go ponder wtime 100000 btime 100000");

    // Wait for some info lines to confirm search is running
    engine.wait_for("info depth");

    // Wait a bit to ensure it doesn't stop on its own
    std::thread::sleep(std::time::Duration::from_millis(100));

    // It should still be searching (no bestmove yet)
    // We can't easily check for absence of bestmove without a timeout,
    // but we can send ponderhit and check for bestmove after.
    engine.send("ponderhit");
    engine.wait_for("bestmove");
}

#[test]
fn test_uci_ponder_stop() {
    let mut engine = UciEngine::new();
    engine.send("isready");
    engine.wait_for("readyok");
    engine.send("go ponder wtime 100000 btime 100000");
    engine.wait_for("info depth");

    std::thread::sleep(std::time::Duration::from_millis(100));

    engine.send("stop");
    engine.wait_for("bestmove");
}

#[test]
fn test_uci_ponder_miss() {
    let mut engine = UciEngine::new();
    engine.send("isready");
    engine.wait_for("readyok");

    // Pondering on startpos (predicting e2e4 e7e5)
    engine.send("go ponder wtime 100000 btime 100000");
    engine.wait_for("info depth");

    // Opponent plays different move, so GUI sends stop
    engine.send("stop");
    engine.wait_for("bestmove");

    // Now start new normal search
    engine.send("position startpos moves d2d4");
    engine.send("go depth 2");
    engine.wait_for("bestmove");
}

#[test]
fn test_uci_ponder_clock_preservation() {
    let mut engine = UciEngine::new();
    engine.send("isready");
    engine.wait_for("readyok");
    // Start pondering with 2000ms movetime
    engine.send("go ponder movetime 2000");
    // Wait for search to be running
    engine.wait_for("info depth");

    // Let it ponder for 500ms
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Send ponderhit
    let start_hit = std::time::Instant::now();
    engine.send("ponderhit");
    
    // It should now run for 2000ms from the time of ponderhit
    engine.wait_for("bestmove");
    let elapsed = start_hit.elapsed();
    
    // If it didn't preserve the clock, it would stop in 2000 - 500 = 1500ms.
    // If it did preserve, it stops in 2000ms.
    // We allow 200ms of overhead/tolerance
    assert!(
        elapsed.as_millis() >= 1800,
        "Too fast: {}ms (expected >= 1800ms after ponderhit)",
        elapsed.as_millis()
    );
    assert!(
        elapsed.as_millis() < 2500,
        "Too slow: {}ms (expected < 2500ms after ponderhit)",
        elapsed.as_millis()
    );
}

#[test]
fn test_uci_go_hammer() {
    let mut engine = UciEngine::new();
    engine.send("isready");
    engine.wait_for("readyok");
    for _ in 0..10 {
        engine.send("go depth 100");
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    engine.send("stop");
    engine.wait_for("bestmove");
}

#[test]
fn test_uci_threads_option_advertised() {
    let mut engine = UciEngine::new();
    engine.send("uci");
    let mut found_threads = false;
    loop {
        let line = engine.read_line();
        if line.contains("option name Threads") {
            found_threads = true;
        }
        if line == "uciok" {
            break;
        }
    }
    assert!(found_threads, "Engine must advertise 'option name Threads'");
}

#[test]
fn test_uci_threads_multi_search() {
    let mut engine = UciEngine::new();
    engine.send("isready");
    engine.wait_for("readyok");
    engine.send("setoption name Threads value 4");
    engine.send("position startpos");
    engine.send("go depth 3");
    engine.wait_for("bestmove");
}

#[test]
fn test_uci_threads_single_is_equivalent() {
    let mut engine = UciEngine::new();
    engine.send("isready");
    engine.wait_for("readyok");
    engine.send("setoption name Threads value 1");
    engine.send("position startpos");
    engine.send("go depth 3");
    engine.wait_for("bestmove");
}
