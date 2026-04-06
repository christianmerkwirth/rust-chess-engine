use crate::types::Move;
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TTFlag {
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TTData {
    pub depth: i8,
    pub score: i16,
    pub flag: TTFlag,
    pub best_move: Move,
    pub gen: u8,
}

impl TTData {
    fn pack(&self) -> u64 {
        let score = (self.score as u16) as u64;
        let mv = (self.best_move.0) as u64;
        let depth = (self.depth as u8) as u64;
        let flag = match self.flag {
            TTFlag::Exact => 0,
            TTFlag::LowerBound => 1,
            TTFlag::UpperBound => 2,
        } as u64;
        let gen = self.gen as u64;

        score | (mv << 16) | (depth << 32) | (flag << 40) | (gen << 48)
    }

    fn unpack(data: u64) -> Self {
        let score = (data & 0xFFFF) as i16;
        let mv = ((data >> 16) & 0xFFFF) as u16;
        let depth = ((data >> 32) & 0xFF) as i8;
        let flag = match (data >> 40) & 0x3 {
            1 => TTFlag::LowerBound,
            2 => TTFlag::UpperBound,
            _ => TTFlag::Exact,
        };
        let gen = ((data >> 48) & 0xFF) as u8;

        TTData {
            depth,
            score,
            flag,
            best_move: Move(mv),
            gen,
        }
    }
}

/// A Transposition Table entry.
/// We use two 64-bit words and a sequence counter to ensure consistency
/// during concurrent access.
struct TTEntry {
    /// word0 = hash ^ data (XOR trick for key storage)
    word0: AtomicU64,
    /// word1 = data
    word1: AtomicU64,
}

pub struct TranspositionTable {
    entries: Box<[TTEntry]>,
    mask: usize,
    generation: AtomicU8,
}

impl TranspositionTable {
    pub fn new(size_mb: usize) -> Self {
        let entry_size = std::mem::size_of::<TTEntry>();
        let n_entries = (size_mb * 1024 * 1024) / entry_size;
        let n_entries = n_entries.next_power_of_two();
        let mut entries = Vec::with_capacity(n_entries);
        for _ in 0..n_entries {
            entries.push(TTEntry {
                word0: AtomicU64::new(0),
                word1: AtomicU64::new(0),
            });
        }
        Self {
            entries: entries.into_boxed_slice(),
            mask: n_entries - 1,
            generation: AtomicU8::new(0),
        }
    }

    pub fn new_search(&self) {
        self.generation.fetch_add(1, Ordering::Relaxed);
    }

    pub fn probe(&self, hash: u64) -> Option<TTData> {
        let index = (hash as usize) & self.mask;
        let entry = &self.entries[index];

        // The XOR trick: read word1 then word0.
        // If they XOR to the hash, the data is valid for this hash.
        // We use Acquire to ensure we see the correct state of both words.
        let word1 = entry.word1.load(Ordering::Acquire);
        let word0 = entry.word0.load(Ordering::Acquire);

        if word0 ^ word1 == hash {
            Some(TTData::unpack(word1))
        } else {
            None
        }
    }

    pub fn store(&self, hash: u64, mut data: TTData) {
        let index = (hash as usize) & self.mask;
        let entry = &self.entries[index];

        let current_gen = self.generation.load(Ordering::Relaxed);
        data.gen = current_gen;

        let old_word1 = entry.word1.load(Ordering::Relaxed);
        let old_word0 = entry.word0.load(Ordering::Relaxed);
        let old_hash = old_word0 ^ old_word1;

        // Replacement logic:
        // 1. If entry is empty, replace.
        // 2. If it's the same hash, always replace (update with newer depth/score/move).
        //    Actually, usually for same hash we only replace if new depth >= old depth?
        //    Wait, for same hash we should probably always replace if it's from a newer search,
        //    or if it has more depth.
        // 3. If it's a different hash, replace if:
        //    a. Old entry is from an older generation.
        //    b. New depth is greater or equal to old depth.

        let mut replace = true;
        if old_hash != 0 {
            let old_data = TTData::unpack(old_word1);
            if old_hash != hash {
                // Different position
                if old_data.gen == current_gen && data.depth < old_data.depth {
                    replace = false;
                }
            }
            // If same hash, we always replace to keep the entry fresh.
        }

        if replace {
            let packed = data.pack();
            // The XOR trick: write word1 then word0.
            // We use Release to ensure that the order is preserved for other threads.
            entry.word1.store(packed, Ordering::Release);
            entry.word0.store(hash ^ packed, Ordering::Release);
        }
    }

    pub fn clear(&self) {
        for entry in self.entries.iter() {
            entry.word0.store(0, Ordering::Relaxed);
            entry.word1.store(0, Ordering::Relaxed);
        }
    }

    pub fn hashfull(&self) -> u32 {
        let mut occupied = 0;
        let sample_size = self.entries.len().min(1000);
        for i in 0..sample_size {
            if self.entries[i].word0.load(Ordering::Relaxed) != 0 {
                occupied += 1;
            }
        }
        occupied * 1000 / sample_size as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Square;

    #[test]
    fn test_tt_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<TranspositionTable>();
    }

    #[test]
    fn test_tt_roundtrip() {
        let tt = TranspositionTable::new(1);
        let hash = 0x123456789ABCDEF0;
        let data = TTData {
            depth: 10,
            score: 1500,
            flag: TTFlag::Exact,
            best_move: Move::new(Square(0), Square(63), 0, 0),
            gen: 0,
        };

        tt.store(hash, data);
        let probed = tt.probe(hash).expect("Probe failed");
        assert_eq!(probed, data);
    }

    #[test]
    fn test_tt_overwrite() {
        let tt = TranspositionTable::new(1);
        let hash = 0x123456789ABCDEF0;
        let data1 = TTData {
            depth: 5,
            score: 100,
            flag: TTFlag::LowerBound,
            best_move: Move::NONE,
            gen: 0,
        };
        let data2 = TTData {
            depth: 10,
            score: 200,
            flag: TTFlag::Exact,
            best_move: Move::NONE,
            gen: 0,
        };

        tt.store(hash, data1);
        tt.store(hash, data2);
        let probed = tt.probe(hash).expect("Probe failed");
        assert_eq!(probed, data2);
    }

    #[test]
    fn test_tt_clear() {
        let tt = TranspositionTable::new(1);
        let hash = 0x123456789ABCDEF0;
        let data = TTData {
            depth: 10,
            score: 1500,
            flag: TTFlag::Exact,
            best_move: Move::NONE,
            gen: 0,
        };

        tt.store(hash, data);
        tt.clear();
        assert!(tt.probe(hash).is_none());
    }

    #[test]
    fn test_tt_replacement_and_aging() {
        let tt = TranspositionTable::new(1);
        let mask = tt.mask;
        let hash1 = 0;
        let hash2 = (mask + 1) as u64;

        // Store shallow entry in gen 0
        tt.store(hash1, TTData { depth: 5, score: 100, flag: TTFlag::Exact, best_move: Move::NONE, gen: 0 });
        assert_eq!(tt.probe(hash1).unwrap().depth, 5);

        // Store deeper entry for DIFFERENT hash in gen 0 - should replace because depth is higher
        tt.store(hash2, TTData { depth: 10, score: 200, flag: TTFlag::Exact, best_move: Move::NONE, gen: 0 });
        assert!(tt.probe(hash1).is_none());
        assert_eq!(tt.probe(hash2).unwrap().depth, 10);

        // Store shallower entry for DIFFERENT hash in gen 0 - should NOT replace because depth is lower
        tt.store(hash1, TTData { depth: 5, score: 300, flag: TTFlag::Exact, best_move: Move::NONE, gen: 0 });
        assert!(tt.probe(hash1).is_none());
        assert_eq!(tt.probe(hash2).unwrap().depth, 10);

        // Advance generation
        tt.new_search();

        // Store shallower entry for DIFFERENT hash in gen 1 - should replace because gen is newer
        tt.store(hash1, TTData { depth: 2, score: 400, flag: TTFlag::Exact, best_move: Move::NONE, gen: 0 });
        assert_eq!(tt.probe(hash1).unwrap().depth, 2);
        assert!(tt.probe(hash2).is_none());
    }
}
