use std::io::{self, BufRead, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use crate::board::Position;
use crate::book::PolyglotBook;
use crate::search::smp::ThreadPool;
use crate::search::tt::TranspositionTable;
use crate::search::SearchInfo;
use crate::tablebase::SyzygyTablebase;
use crate::time::{self, GoParams};

pub struct Engine {
    pub pos: Position,
    pub tt: Arc<TranspositionTable>,
    pub stop: Arc<AtomicBool>,
    pub pondering: Arc<AtomicBool>,
    pub search_handle: Option<JoinHandle<()>>,
    pub book: Option<PolyglotBook>,
    pub tablebase: Option<Arc<SyzygyTablebase>>,
    pub pool: ThreadPool,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    pub fn new() -> Self {
        Self {
            pos: Position::startpos(),
            tt: Arc::new(TranspositionTable::new(64)), // Default 64MB
            stop: Arc::new(AtomicBool::new(false)),
            pondering: Arc::new(AtomicBool::new(false)),
            search_handle: None,
            book: None,
            tablebase: None,
            pool: ThreadPool::new(1),
        }
    }

    pub fn reset_tt(&mut self, size_mb: usize) {
        self.tt = Arc::new(TranspositionTable::new(size_mb));
    }
}

pub fn uci_loop() {
    crate::movegen::magics::init();
    let mut engine = Engine::new();
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    while let Some(Ok(line)) = lines.next() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "uci" => {
                println!("id name ChessEngine");
                println!("id author Gemini CLI");
                println!("option name Hash type spin default 64 min 1 max 4096");
                println!("option name Threads type spin default 1 min 1 max 256");
                println!("option name Ponder type check default false");
                println!("option name BookPath type string default <empty>");
                println!("option name SyzygyPath type string default <empty>");
                println!("uciok");
            }
            "isready" => println!("readyok"),
            "ucinewgame" => {
                engine.tt.clear();
                engine.pos = Position::startpos();
            }
            "setoption" => {
                handle_setoption(&mut engine, &parts);
            }
            "position" => {
                parse_position(&mut engine, &parts);
            }
            "go" => {
                parse_go(&mut engine, &parts);
            }
            "stop" => {
                engine.stop.store(true, Ordering::Relaxed);
                engine.pondering.store(false, Ordering::Relaxed);
            }
            "ponderhit" => {
                engine.pondering.store(false, Ordering::Relaxed);
            }
            "quit" => {
                engine.stop.store(true, Ordering::Relaxed);
                engine.pondering.store(false, Ordering::Relaxed);
                break;
            }
            _ => {}
        }
        io::stdout().flush().unwrap();
    }

    // Wait for search thread to finish before exiting
    if let Some(handle) = engine.search_handle.take() {
        let _ = handle.join();
    }
}

fn parse_position(engine: &mut Engine, parts: &[&str]) {
    if parts.len() < 2 {
        return;
    }

    let mut current_idx = 1;
    if parts[1] == "startpos" {
        engine.pos = Position::startpos();
        current_idx = 2;
    } else if parts[1] == "fen" {
        let mut fen_parts = Vec::new();
        current_idx = 2;
        while current_idx < parts.len() && parts[current_idx] != "moves" {
            fen_parts.push(parts[current_idx]);
            current_idx += 1;
        }
        let fen = fen_parts.join(" ");
        if let Ok(pos) = Position::from_fen(&fen) {
            engine.pos = pos;
        }
    }

    if current_idx < parts.len() && parts[current_idx] == "moves" {
        current_idx += 1;
        while current_idx < parts.len() {
            if let Some(mv) = engine.pos.parse_move(parts[current_idx]) {
                engine.pos.make_move(mv);
            }
            current_idx += 1;
        }
    }
}

fn handle_setoption(engine: &mut Engine, parts: &[&str]) {
    // Expect: setoption name <name> value <value>
    // Find "name" and "value" tokens
    let name_idx = parts.iter().position(|&p| p == "name");
    let value_idx = parts.iter().position(|&p| p == "value");
    let (Some(ni), Some(vi)) = (name_idx, value_idx) else {
        return;
    };
    if vi <= ni {
        return;
    }

    let name: String = parts[ni + 1..vi].join(" ");
    let value: String = parts[vi + 1..].join(" ");

    match name.as_str() {
        "Hash" => {
            if let Ok(size_mb) = value.parse::<usize>() {
                engine.reset_tt(size_mb);
            }
        }
        "Threads" => {
            if let Ok(n) = value.parse::<usize>() {
                engine.pool.resize(n);
            }
        }
        "BookPath" => {
            if value.is_empty() || value == "<empty>" {
                engine.book = None;
            } else {
                match PolyglotBook::open(&value) {
                    Ok(book) => engine.book = Some(book),
                    Err(e) => eprintln!("info string BookPath error: {:?}", e),
                }
            }
        }
        "SyzygyPath" => {
            if value.is_empty() || value == "<empty>" {
                engine.tablebase = None;
            } else {
                match SyzygyTablebase::new(Path::new(&value)) {
                    Ok(tb) => engine.tablebase = Some(Arc::new(tb)),
                    Err(e) => eprintln!("info string SyzygyPath error: {:?}", e),
                }
            }
        }
        _ => {}
    }
}

fn parse_go(engine: &mut Engine, parts: &[&str]) {
    // Book probe: if the position is in the opening book, return immediately.
    if let Some(ref book) = engine.book {
        if let Some(book_move) = book.probe(&engine.pos) {
            println!("bestmove {}", book_move.to_uci());
            io::stdout().flush().unwrap();
            return;
        }
    }

    let mut params = GoParams::default();
    let mut i = 1;
    while i < parts.len() {
        match parts[i] {
            "wtime" => {
                if i + 1 < parts.len() {
                    params.wtime = parts[i + 1].parse().ok();
                    i += 1;
                }
            }
            "btime" => {
                if i + 1 < parts.len() {
                    params.btime = parts[i + 1].parse().ok();
                    i += 1;
                }
            }
            "winc" => {
                if i + 1 < parts.len() {
                    params.winc = parts[i + 1].parse().ok();
                    i += 1;
                }
            }
            "binc" => {
                if i + 1 < parts.len() {
                    params.binc = parts[i + 1].parse().ok();
                    i += 1;
                }
            }
            "movestogo" => {
                if i + 1 < parts.len() {
                    params.movestogo = parts[i + 1].parse().ok();
                    i += 1;
                }
            }
            "depth" => {
                if i + 1 < parts.len() {
                    params.depth = parts[i + 1].parse().ok();
                    i += 1;
                }
            }
            "nodes" => {
                if i + 1 < parts.len() {
                    params.nodes = parts[i + 1].parse().ok();
                    i += 1;
                }
            }
            "movetime" => {
                if i + 1 < parts.len() {
                    params.movetime = parts[i + 1].parse().ok();
                    i += 1;
                }
            }
            "infinite" => params.infinite = true,
            "ponder" => params.ponder = true,
            _ => {}
        }
        i += 1;
    }

    let limits = time::allocate_time(&params, engine.pos.side_to_move());

    // Start search in a separate thread
    engine.stop.store(false, Ordering::Relaxed);
    engine.pondering.store(params.ponder, Ordering::Relaxed);
    let pos = engine.pos.clone();
    let tt = Arc::clone(&engine.tt);
    let stop = Arc::clone(&engine.stop);
    let pondering = Arc::clone(&engine.pondering);
    let tablebase = engine.tablebase.as_ref().map(Arc::clone);
    let num_threads = engine.pool.num_threads;

    engine.search_handle = Some(thread::spawn(move || {
        let pool = ThreadPool::new(num_threads);
        let result = pool.search(&pos, tt, &limits, stop, pondering, tablebase, |info| {
            print_info(&info);
        });

        let mut output = format!("bestmove {}", result.best_move.to_uci());
        if let Some(pm) = result.ponder_move {
            output.push_str(&format!(" ponder {}", pm.to_uci()));
        }
        println!("{}", output);
        io::stdout().flush().unwrap();
    }));
}

fn print_info(info: &SearchInfo) {
    let score_str = if info.score.abs() > 29000 {
        let mate_in = (30000 - info.score.abs() + 1) / 2;
        format!("mate {}", if info.score > 0 { mate_in } else { -mate_in })
    } else {
        format!("cp {}", info.score)
    };

    let pv_str: Vec<String> = info.pv.iter().map(|m| m.to_uci()).collect();

    println!(
        "info depth {} score {} nodes {} time {} nps {} pv {}",
        info.depth,
        score_str,
        info.nodes,
        info.time,
        if info.time > 0 {
            info.nodes * 1000 / info.time as u64
        } else {
            0
        },
        pv_str.join(" ")
    );
}
