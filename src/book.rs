use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use rand::Rng;
use crate::board::Position;
use crate::types::{Move, Piece, Square};

#[derive(Debug)]
pub enum BookError {
    Io(std::io::Error),
    InvalidFile,
}

impl From<std::io::Error> for BookError {
    fn from(err: std::io::Error) -> Self {
        BookError::Io(err)
    }
}

pub struct PolyglotBook {
    file: File,
    num_entries: u64,
}

#[derive(Clone, Copy, Debug)]
struct BookEntry {
    key: u64,
    raw_move: u16,
    weight: u16,
    _learn: u32,
}

impl PolyglotBook {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, BookError> {
        let file = File::open(path)?;
        let metadata = file.metadata()?;
        let len = metadata.len();
        if len % 16 != 0 {
            return Err(BookError::InvalidFile);
        }
        Ok(PolyglotBook {
            file,
            num_entries: len / 16,
        })
    }

    pub fn probe(&self, _pos: &Position) -> Option<Move> {
        // STUB: always returns None — tests will fail until real implementation added
        None
    }

    fn read_entry(&self, idx: u64) -> Result<BookEntry, std::io::Error> {
        let mut file = &self.file;
        file.seek(SeekFrom::Start(idx * 16))?;
        let mut buffer = [0u8; 16];
        file.read_exact(&mut buffer)?;
        Ok(BookEntry {
            key: u64::from_be_bytes(buffer[0..8].try_into().unwrap()),
            raw_move: u16::from_be_bytes(buffer[8..10].try_into().unwrap()),
            weight: u16::from_be_bytes(buffer[10..12].try_into().unwrap()),
            _learn: u32::from_be_bytes(buffer[12..16].try_into().unwrap()),
        })
    }
}

/// Compute the Polyglot Zobrist hash for `pos` using shakmaty's implementation,
/// which matches the standard Polyglot random table exactly.
pub fn polyglot_hash(_pos: &Position) -> u64 {
    // STUB: returns wrong value — tests will fail until real implementation added
    0
}

fn decode_polyglot_move(_raw: u16, _pos: &Position) -> Option<Move> {
    // STUB: returns None — tests will fail until real implementation added
    None
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Position;
    use std::io::Write as IoWrite;

    /// Build a 16-byte Polyglot book entry.
    /// raw_move bit layout: to_file[2:0] | to_rank[5:3] | from_file[8:6] | from_rank[11:9] | promo[14:12]
    fn make_entry(key: u64, from_file: u8, from_rank: u8, to_file: u8, to_rank: u8, promo: u8, weight: u16) -> Vec<u8> {
        let raw_move: u16 = (to_file as u16)
            | ((to_rank as u16) << 3)
            | ((from_file as u16) << 6)
            | ((from_rank as u16) << 9)
            | ((promo as u16) << 12);
        let mut entry = Vec::with_capacity(16);
        entry.extend_from_slice(&key.to_be_bytes());
        entry.extend_from_slice(&raw_move.to_be_bytes());
        entry.extend_from_slice(&weight.to_be_bytes());
        entry.extend_from_slice(&0u32.to_be_bytes());
        entry
    }

    fn tmp(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(name)
    }

    // -----------------------------------------------------------------------
    // polyglot_hash tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_polyglot_hash_startpos() {
        let pos = Position::startpos();
        // Standard Polyglot Zobrist hash for the starting position.
        assert_eq!(polyglot_hash(&pos), 0x463b96181691fc9c);
    }

    #[test]
    fn test_polyglot_hash_after_e4() {
        let mut pos = Position::startpos();
        let moves = crate::movegen::generate_moves(&pos);
        let e4 = (&moves).into_iter().find(|m| m.to_uci() == "e2e4").unwrap();
        pos.make_move(e4);
        // Known hash from shakmaty's test suite
        assert_eq!(polyglot_hash(&pos), 0x823c9b50fd114196);
    }

    #[test]
    fn test_polyglot_hash_differs_by_position() {
        let pos1 = Position::startpos();
        let pos2 = Position::from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1").unwrap();
        assert_ne!(polyglot_hash(&pos1), polyglot_hash(&pos2));
    }

    // -----------------------------------------------------------------------
    // PolyglotBook::open tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_book_open_invalid_size() {
        let path = tmp("test_book_invalid.bin");
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(&[0u8; 17]).unwrap(); // 17 bytes is not divisible by 16
        drop(f);
        assert!(matches!(PolyglotBook::open(&path), Err(BookError::InvalidFile)));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_book_open_empty_file_is_valid() {
        let path = tmp("test_book_empty.bin");
        std::fs::File::create(&path).unwrap(); // 0 bytes — valid (0 % 16 == 0)
        assert!(PolyglotBook::open(&path).is_ok());
        std::fs::remove_file(&path).ok();
    }

    // -----------------------------------------------------------------------
    // PolyglotBook::probe tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_book_probe_unknown_position_returns_none() {
        let path = tmp("test_book_probe_unknown.bin");
        // Write one entry for a key that is NOT the startpos hash
        let mut data = make_entry(0xDEADBEEF_00000000, 4, 1, 4, 3, 0, 100);
        std::fs::write(&path, &data).unwrap();
        let book = PolyglotBook::open(&path).unwrap();
        let pos = Position::startpos();
        assert!(book.probe(&pos).is_none());
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_book_probe_known_position_returns_e2e4() {
        // Create a book with a single entry: startpos → e2e4
        let pos = Position::startpos();
        // Use the EXPECTED hash (0x463b96181691fc9c); the test implicitly verifies
        // that polyglot_hash(&startpos) returns this value.
        let key: u64 = 0x463b96181691fc9c;
        let path = tmp("test_book_known.bin");
        // e2e4: from=(file=4,rank=1) to=(file=4,rank=3)
        std::fs::write(&path, &make_entry(key, 4, 1, 4, 3, 0, 100)).unwrap();
        let book = PolyglotBook::open(&path).unwrap();
        let result = book.probe(&pos);
        assert!(result.is_some(), "probe should find book move for startpos");
        assert_eq!(result.unwrap().to_uci(), "e2e4");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_book_probe_empty_book_returns_none() {
        let path = tmp("test_book_probe_empty.bin");
        std::fs::File::create(&path).unwrap(); // 0 bytes
        let book = PolyglotBook::open(&path).unwrap();
        assert!(book.probe(&Position::startpos()).is_none());
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_book_probe_weighted_selection() {
        // Write two moves for startpos: e2e4 (weight=900) and d2d4 (weight=100).
        // Over 1000 probes, e2e4 should appear approximately 9× more often than d2d4.
        let key: u64 = 0x463b96181691fc9c;
        let path = tmp("test_book_weights.bin");
        let mut data = Vec::new();
        // e2e4: from=(4,1) to=(4,3) — must sort entries by key (same key here)
        data.extend_from_slice(&make_entry(key, 4, 1, 4, 3, 0, 900));
        // d2d4: from=(3,1) to=(3,3)
        data.extend_from_slice(&make_entry(key, 3, 1, 3, 3, 0, 100));
        std::fs::write(&path, &data).unwrap();
        let book = PolyglotBook::open(&path).unwrap();
        let pos = Position::startpos();
        let mut e4_count = 0u32;
        let mut d4_count = 0u32;
        for _ in 0..200 {
            match book.probe(&pos).map(|m| m.to_uci()) {
                Some(s) if s == "e2e4" => e4_count += 1,
                Some(s) if s == "d2d4" => d4_count += 1,
                _ => {}
            }
        }
        assert!(e4_count > d4_count, "higher-weight move should appear more often (e4={}, d4={})", e4_count, d4_count);
        std::fs::remove_file(&path).ok();
    }

    // -----------------------------------------------------------------------
    // decode_polyglot_move tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_decode_polyglot_castling_white_kingside() {
        let pos = Position::startpos(); // king at e1=Square(4)
        // Polyglot encodes WK castling as e1→h1: to_file=7,to_rank=0,from_file=4,from_rank=0
        let raw: u16 = 7 | (0 << 3) | (4 << 6) | (0 << 9); // e1h1
        let mv = decode_polyglot_move(raw, &pos);
        assert!(mv.is_some(), "should decode WK castling");
        let mv = mv.unwrap();
        assert!(mv.is_castling(), "should be castling flag");
        assert_eq!(mv.from_sq(), Square(4), "from e1");
        assert_eq!(mv.to_sq(), Square(6), "to g1");
    }

    #[test]
    fn test_decode_polyglot_castling_white_queenside() {
        let pos = Position::startpos(); // king at e1=Square(4)
        // Polyglot encodes WQ castling as e1→a1: to_file=0,to_rank=0,from_file=4,from_rank=0
        let raw: u16 = 0 | (0 << 3) | (4 << 6) | (0 << 9); // e1a1
        let mv = decode_polyglot_move(raw, &pos);
        assert!(mv.is_some(), "should decode WQ castling");
        let mv = mv.unwrap();
        assert!(mv.is_castling(), "should be castling flag");
        assert_eq!(mv.from_sq(), Square(4), "from e1");
        assert_eq!(mv.to_sq(), Square(2), "to c1");
    }

    #[test]
    fn test_decode_polyglot_castling_black_kingside() {
        let pos = Position::from_fen("rnbqk2r/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1").unwrap();
        // Black king at e8=Square(60), Polyglot encodes BK castling as e8→h8
        let raw: u16 = 7 | (7 << 3) | (4 << 6) | (7 << 9); // e8h8
        let mv = decode_polyglot_move(raw, &pos);
        assert!(mv.is_some(), "should decode BK castling");
        let mv = mv.unwrap();
        assert!(mv.is_castling());
        assert_eq!(mv.from_sq(), Square(60), "from e8");
        assert_eq!(mv.to_sq(), Square(62), "to g8");
    }

    #[test]
    fn test_decode_polyglot_castling_black_queenside() {
        let pos = Position::from_fen("r3kbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1").unwrap();
        let raw: u16 = 0 | (7 << 3) | (4 << 6) | (7 << 9); // e8a8
        let mv = decode_polyglot_move(raw, &pos);
        assert!(mv.is_some(), "should decode BQ castling");
        let mv = mv.unwrap();
        assert!(mv.is_castling());
        assert_eq!(mv.from_sq(), Square(60), "from e8");
        assert_eq!(mv.to_sq(), Square(58), "to c8");
    }

    #[test]
    fn test_decode_polyglot_normal_pawn_move() {
        let pos = Position::startpos();
        // e2e4: from=(4,1) to=(4,3) no promo
        let raw: u16 = 4 | (3 << 3) | (4 << 6) | (1 << 9);
        let mv = decode_polyglot_move(raw, &pos);
        assert!(mv.is_some());
        let mv = mv.unwrap();
        assert_eq!(mv.to_uci(), "e2e4");
        assert!(!mv.is_castling());
        assert!(!mv.is_promotion());
    }

    #[test]
    fn test_decode_polyglot_promotion_to_queen() {
        // White pawn on a7, promotes to queen: a7a8q = from=(0,6) to=(0,7) promo=4
        let pos = Position::from_fen("8/P7/8/8/8/8/8/4K1k1 w - - 0 1").unwrap();
        let raw: u16 = 0 | (7 << 3) | (0 << 6) | (6 << 9) | (4 << 12);
        let mv = decode_polyglot_move(raw, &pos);
        assert!(mv.is_some());
        let mv = mv.unwrap();
        assert!(mv.is_promotion());
        assert_eq!(mv.promotion_piece(), Piece::Queen);
    }
}
