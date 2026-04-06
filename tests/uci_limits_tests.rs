use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio, Child};

fn spawn_engine() -> Child {
    Command::new("./target/debug/chess_engine")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn engine")
}

fn send(child: &mut Child, msg: &str) {
    let stdin = child.stdin.as_mut().unwrap();
    writeln!(stdin, "{}", msg).unwrap();
    stdin.flush().unwrap();
}

#[test]
fn test_uci_author_identity() {
    let mut child = spawn_engine();
    send(&mut child, "uci");
    
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);
    let mut found_author = false;
    let mut line = String::new();
    while reader.read_line(&mut line).unwrap() > 0 {
        if line.contains("id author Christian Merkwirth") {
            found_author = true;
        }
        if line.trim() == "uciok" {
            break;
        }
        line.clear();
    }
    assert!(found_author, "Author identity not found or incorrect");
    child.kill().unwrap();
}

#[test]
fn test_position_atomicity() {
    let mut child = spawn_engine();
    send(&mut child, "position startpos moves e2e4");
    send(&mut child, "position startpos moves d2d4 z9z9"); // Should fail and stay at e2e4
    send(&mut child, "go depth 1");
    
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();
    let mut info_invalid_move = false;
    while reader.read_line(&mut line).unwrap() > 0 {
        if line.contains("info string invalid move") {
            info_invalid_move = true;
        }
        if line.starts_with("bestmove") {
            break;
        }
        line.clear();
    }
    
    assert!(info_invalid_move, "Should have emitted info string invalid move");
    child.kill().unwrap();
}

#[test]
fn test_go_nodes_limit() {
    let mut child = spawn_engine();
    // A reasonably small nodes limit that should be hit quickly
    send(&mut child, "go nodes 5000");
    
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();
    let mut total_nodes = 0;
    while reader.read_line(&mut line).unwrap() > 0 {
        if line.starts_with("info") {
            if let Some(nodes_idx) = line.split_whitespace().position(|p| p == "nodes") {
                if let Some(n) = line.split_whitespace().nth(nodes_idx + 1) {
                    total_nodes = n.parse::<u64>().unwrap_or(0);
                }
            }
        }
        if line.starts_with("bestmove") {
            break;
        }
        line.clear();
    }
    
    assert!(total_nodes >= 5000, "Should have searched at least 5000 nodes, got {}", total_nodes);
    // It shouldn't be TOO much more. We check every 2048 nodes.
    assert!(total_nodes < 10000, "Should have stopped near 5000 nodes, got {}", total_nodes);
    
    child.kill().unwrap();
}

#[test]
fn test_bestmove_0000_checkmate() {
    let mut child = spawn_engine();
    // Position where black is checkmated:
    send(&mut child, "position fen R3k3/8/4K3/8/8/8/8/8 b - - 0 1");
    send(&mut child, "go depth 1");
    
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();
    let mut found_0000 = false;
    while reader.read_line(&mut line).unwrap() > 0 {
        if line.trim() == "bestmove 0000" {
            found_0000 = true;
            break;
        }
        line.clear();
    }
    assert!(found_0000);
    child.kill().unwrap();
}

#[test]
fn test_go_depth_limit() {
    let mut child = spawn_engine();
    send(&mut child, "go depth 3");
    
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();
    let mut max_depth = 0;
    while reader.read_line(&mut line).unwrap() > 0 {
        if line.starts_with("info") {
            if let Some(depth_idx) = line.split_whitespace().position(|p| p == "depth") {
                if let Some(d) = line.split_whitespace().nth(depth_idx + 1) {
                    let depth = d.parse::<i32>().unwrap_or(0);
                    if depth > max_depth {
                        max_depth = depth;
                    }
                }
            }
        }
        if line.starts_with("bestmove") {
            break;
        }
        line.clear();
    }
    
    assert!(max_depth <= 3, "Should not have exceeded depth 3, got {}", max_depth);
    assert!(max_depth > 0, "Should have reached at least some depth");
    
    child.kill().unwrap();
}
