use chess_engine::board::Position;
use chess_engine::movegen::{magics, perft};

#[test]
fn test_perft_startpos() {
    magics::init();
    let pos = Position::startpos();

    assert_eq!(perft(&pos, 1), 20);
    assert_eq!(perft(&pos, 2), 400);
    assert_eq!(perft(&pos, 3), 8_902);
    assert_eq!(perft(&pos, 4), 197_281);
    assert_eq!(perft(&pos, 5), 4_865_609);
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn test_perft_startpos_depth_6() {
    magics::init();
    let pos = Position::startpos();
    assert_eq!(perft(&pos, 6), 119_060_324);
}

#[test]
fn test_perft_kiwipete() {
    magics::init();
    // Position 2: Kiwipete
    let pos =
        Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
            .unwrap();

    assert_eq!(perft(&pos, 1), 48);
    assert_eq!(perft(&pos, 2), 2_039);
    assert_eq!(perft(&pos, 3), 97_862);
}

#[test]
fn test_perft_pos3() {
    magics::init();
    let pos = Position::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap();
    assert_eq!(perft(&pos, 1), 14);
    assert_eq!(perft(&pos, 2), 191);
    assert_eq!(perft(&pos, 3), 2_812);
    assert_eq!(perft(&pos, 4), 43_238);
}

#[test]
fn test_perft_pos4() {
    magics::init();
    let pos =
        Position::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1")
            .unwrap();
    assert_eq!(perft(&pos, 1), 6);
    assert_eq!(perft(&pos, 2), 264);
    assert_eq!(perft(&pos, 3), 9_467);
}

#[test]
fn test_perft_pos5() {
    magics::init();
    let pos =
        Position::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8").unwrap();
    assert_eq!(perft(&pos, 1), 44);
    assert_eq!(perft(&pos, 2), 1_486);
    assert_eq!(perft(&pos, 3), 62_379);
}

#[test]
fn test_perft_pos6() {
    magics::init();
    let pos = Position::from_fen(
        "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
    )
    .unwrap();
    assert_eq!(perft(&pos, 1), 46);
    assert_eq!(perft(&pos, 2), 2_079);
    assert_eq!(perft(&pos, 3), 89_890);
}
