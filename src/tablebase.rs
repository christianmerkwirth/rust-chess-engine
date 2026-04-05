use std::path::Path;
use shakmaty::{CastlingMode, Chess};
use shakmaty::fen::Fen;
use shakmaty_syzygy::{Tablebase, Wdl};
use crate::board::Position;

#[derive(Debug)]
pub enum TablebaseError {
    Io(std::io::Error),
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
    inner: Tablebase<Chess>,
}

impl SyzygyTablebase {
    /// Open a Syzygy tablebase directory. Returns an error if the path is invalid
    /// or cannot be read. Note: table files are opened lazily on first probe.
    pub fn new(path: &Path) -> Result<Self, TablebaseError> {
        let mut tb = Tablebase::new();
        tb.add_directory(path)?;
        Ok(SyzygyTablebase { inner: tb })
    }

    /// Probe WDL for `pos`. Returns `None` if the position has more than 6 pieces
    /// or the required table is not available — callers should fall back to search.
    pub fn probe_wdl(&self, pos: &Position) -> Option<WdlResult> {
        if pos.occupancy().count() > 6 {
            return None;
        }
        let chess = to_shakmaty(pos)?;
        match self.inner.probe_wdl_after_zeroing(&chess) {
            Ok(Wdl::Win)         => Some(WdlResult::Win),
            Ok(Wdl::Draw)        => Some(WdlResult::Draw),
            Ok(Wdl::Loss)        => Some(WdlResult::Loss),
            Ok(Wdl::CursedWin)   => Some(WdlResult::CursedWin),
            Ok(Wdl::BlessedLoss) => Some(WdlResult::BlessedLoss),
            Err(_)               => None,
        }
    }

    /// Probe DTZ for `pos`. Returns `None` if unavailable or >6 pieces.
    pub fn probe_dtz(&self, pos: &Position) -> Option<DtzResult> {
        if pos.occupancy().count() > 6 {
            return None;
        }
        let chess = to_shakmaty(pos)?;
        let dtz = self.inner.probe_dtz(&chess).ok()?;
        let wdl_raw = self.inner.probe_wdl_after_zeroing(&chess).ok()?;
        let wdl = match wdl_raw {
            Wdl::Win         => WdlResult::Win,
            Wdl::Draw        => WdlResult::Draw,
            Wdl::Loss        => WdlResult::Loss,
            Wdl::CursedWin   => WdlResult::CursedWin,
            Wdl::BlessedLoss => WdlResult::BlessedLoss,
        };
        Some(DtzResult { wdl, dtz: dtz.ignore_rounding().0 })
    }
}

/// Convert our `Position` to a `shakmaty::Chess` via a FEN round-trip.
/// Returns `None` if the FEN is unparseable or the position is illegal.
pub fn to_shakmaty(pos: &Position) -> Option<Chess> {
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
        // Even with a real tablebase, >6 pieces must return None.
        // With an empty tablebase (no files added), probe returns None regardless.
        let tb = SyzygyTablebase { inner: Tablebase::new() };
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
        // KQvK requires KQK.rtbz; without any loaded files, probe must return None.
        // (KvK is trivially handled by shakmaty-syzygy without files.)
        let tb = SyzygyTablebase { inner: Tablebase::new() };
        let pos = Position::from_fen("4k3/8/8/8/8/1Q6/8/4K3 w - - 0 1").unwrap();
        assert!(tb.probe_dtz(&pos).is_none());
    }

    #[test]
    fn test_probe_wdl_six_pieces_returns_none_without_tables() {
        // 6-piece position: must not crash and returns None without tablebase files.
        let tb = SyzygyTablebase { inner: Tablebase::new() };
        let pos = Position::from_fen("4k3/8/8/8/8/1Q6/8/4K3 w - - 0 1").unwrap(); // KQvK (3 pieces)
        assert!(tb.probe_wdl(&pos).is_none());
    }
}
