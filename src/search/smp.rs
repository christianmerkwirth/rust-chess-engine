use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;

use crate::board::Position;
use crate::search::tt::TranspositionTable;
use crate::search::{iterative_deepening, SearchInfo, SearchLimits, SearchResult};
use crate::tablebase::SyzygyTablebase;

pub struct ThreadPool {
    pub num_threads: usize,
}

impl ThreadPool {
    pub fn new(num_threads: usize) -> Self {
        ThreadPool {
            num_threads: num_threads.max(1),
        }
    }

    pub fn resize(&mut self, num_threads: usize) {
        self.num_threads = num_threads.max(1);
    }

    /// Run iterative deepening with Lazy SMP.
    #[allow(clippy::too_many_arguments)]
    pub fn search(
        &self,
        pos: &Position,
        tt: Arc<TranspositionTable>,
        limits: &SearchLimits,
        stop: Arc<AtomicBool>,
        pondering: Arc<AtomicBool>,
        tablebase: Option<Arc<SyzygyTablebase>>,
        info_callback: impl Fn(SearchInfo),
    ) -> SearchResult {
        let mut handles = Vec::new();
        let nodes = Arc::new(AtomicU64::new(0));

        // Spawn N-1 helper threads
        for i in 1..self.num_threads {
            let pos_clone = pos.clone();
            let tt_clone = Arc::clone(&tt);
            let limits_clone = limits.clone();
            let stop_clone = Arc::clone(&stop);
            let pondering_clone = Arc::clone(&pondering);
            let nodes_clone = Arc::clone(&nodes);
            let tb_clone = tablebase.as_ref().map(Arc::clone);
            let start_depth = 1 + (i as i32 % 3);

            let handle = thread::spawn(move || {
                iterative_deepening(
                    &pos_clone,
                    &tt_clone,
                    &limits_clone,
                    &stop_clone,
                    &pondering_clone,
                    &nodes_clone,
                    tb_clone,
                    start_depth,
                    |_| {},
                );
            });
            handles.push(handle);
        }

        // Main thread
        let result = iterative_deepening(
            pos,
            &tt,
            limits,
            &stop,
            &pondering,
            &nodes,
            tablebase,
            1,
            info_callback,
        );

        stop.store(true, Ordering::Relaxed);
        for handle in handles {
            let _ = handle.join();
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicBool;

    fn make_pool_search(num_threads: usize) {
        crate::movegen::magics::init();
        let pos = crate::board::Position::startpos();
        let tt = Arc::new(TranspositionTable::new(1));
        let stop = Arc::new(AtomicBool::new(false));
        let pondering = Arc::new(AtomicBool::new(false));
        let limits = SearchLimits {
            depth: Some(3),
            ..Default::default()
        };

        let pool = ThreadPool::new(num_threads);
        let result = pool.search(&pos, tt, &limits, stop, pondering, None, |_| {});

        assert_ne!(
            result.best_move,
            crate::types::Move::NONE,
            "ThreadPool with {} threads must return a valid move",
            num_threads
        );
    }

    #[test]
    fn test_thread_pool_single_thread() {
        make_pool_search(1);
    }

    #[test]
    fn test_thread_pool_four_threads() {
        make_pool_search(4);
    }
}
