use crate::search::SearchLimits;
use crate::types::Color;

#[derive(Default)]
pub struct GoParams {
    pub wtime: Option<u64>,
    pub btime: Option<u64>,
    pub winc: Option<u64>,
    pub binc: Option<u64>,
    pub movestogo: Option<u32>,
    pub depth: Option<i32>,
    pub nodes: Option<u64>,
    pub movetime: Option<u64>,
    pub infinite: bool,
    pub ponder: bool,
}

pub fn allocate_time(params: &GoParams, side: Color) -> SearchLimits {
    let mut limits = SearchLimits {
        depth: params.depth,
        movetime: params.movetime,
        nodes: params.nodes,
        infinite: params.infinite,
        ponder: params.ponder,
    };

    if limits.infinite || limits.depth.is_some() || limits.movetime.is_some() || limits.nodes.is_some() {
        return limits;
    }

    let (time, inc) = match side {
        Color::White => (params.wtime.unwrap_or(0), params.winc.unwrap_or(0)),
        Color::Black => (params.btime.unwrap_or(0), params.binc.unwrap_or(0)),
    };

    if time > 0 {
        let moves_to_go = params.movestogo.unwrap_or(30);

        // Simple time management:
        // Allocate 1/moves_to_go of our time + a fraction of the increment.
        // We also want a safety margin.
        let allocated = time / moves_to_go as u64 + (inc * 3 / 4);

        // Don't spend more than we have, keep a safety margin of 50ms.
        let safety_margin = 50;
        let limit = if allocated > time - safety_margin {
            time.saturating_sub(safety_margin)
        } else {
            allocated
        };

        limits.movetime = Some(limit);
    }

    limits
}
