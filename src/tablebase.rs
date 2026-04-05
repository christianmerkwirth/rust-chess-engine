use std::path::Path;
use crate::board::Position;

#[derive(Debug)]
pub enum TablebaseError {
    Io(std::io::Error),
    InvalidPosition,
}

impl From<std::io::Error> for TablebaseError {
    fn from(err: std::io::Error) -> Self {
        TablebaseError::Io(err)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WdlResult {
    Win,
    Draw,
    Loss,
    CursedWin,
    BlessedLoss,
}

#[derive(Debug, Clone, Copy)]
pub struct DtzResult {
    pub wdl: WdlResult,
    pub dtz: i32,
}

pub struct SyzygyTablebase {
    // STUB: empty placeholder
    _priv: (),
}

impl SyzygyTablebase {
    /// Open a Syzygy tablebase directory. Returns an error if the path is invalid.
    pub fn new(_path: &Path) -> Result<Self, TablebaseError> {
        // STUB: always fails — tests will verify graceful fallback behavior
        Err(TablebaseError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "tablebase not implemented",
        )))
    }

    /// Probe WDL. Returns `None` if position has >6 pieces or tables unavailable.
    pub fn probe_wdl(&self, _pos: &Position) -> Option<WdlResult> {
        None
    }

    /// Probe DTZ. Returns `None` if position has >6 pieces or tables unavailable.
    pub fn probe_dtz(&self, _pos: &Position) -> Option<DtzResult> {
        None
    }
}

/// Convert our Position to a shakmaty::Chess via FEN round-trip.
pub fn to_shakmaty(pos: &Position) -> Option<shakmaty::Chess> {
    use shakmaty::{CastlingMode, Chess};
    use shakmaty::fen::Fen;
    let fen_str = pos.to_fen();
    let fen: Fen = fen_str.parse().ok()?;
    fen.into_position::<Chess>(CastlingMode::Standard).ok()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Position;

    #[test]
    fn test_to_shakmaty_startpos() {
        let pos = Position::startpos();
        let chess = to_shakmaty(&pos);
        assert!(chess.is_some(), "startpos should convert to shakmaty::Chess");
    }

    #[test]
    fn test_to_shakmaty_kk_endgame() {
        let pos = Position::from_fen("4k3/8/8/8/8/8/8/4K3 w - - 0 1").unwrap();
        assert!(to_shakmaty(&pos).is_some());
    }

    #[test]
    fn test_probe_wdl_too_many_pieces_returns_none() {
        // SyzygyTablebase stub always returns None regardless of piece count.
        // The real implementation must also return None for >6-piece positions.
        // We test piece-count gating in the real implementation below.
        // For stub: probe always returns None.
        let tb = SyzygyTablebase { _priv: () };
        let pos = Position::startpos(); // 32 pieces
        assert!(tb.probe_wdl(&pos).is_none());
    }

    #[test]
    fn test_syzygy_new_missing_path_returns_error() {
        let result = SyzygyTablebase::new(Path::new("/nonexistent/tablebase/path"));
        assert!(result.is_err(), "should return error for non-existent directory");
    }

    #[test]
    fn test_probe_dtz_returns_none_when_no_tables() {
        let tb = SyzygyTablebase { _priv: () };
        let pos = Position::from_fen("4k3/8/8/8/8/8/8/4K3 w - - 0 1").unwrap();
        assert!(tb.probe_dtz(&pos).is_none());
    }
}
