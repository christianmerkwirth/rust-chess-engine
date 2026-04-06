pub mod magics;

use crate::board::Position;
use crate::types::{CastlingRights, Color, Move, Piece, Square};

pub struct MoveList {
    moves: [Move; 256],
    count: usize,
}

impl Default for MoveList {
    fn default() -> Self {
        Self::new()
    }
}

impl MoveList {
    pub fn new() -> Self {
        Self {
            moves: [Move::NONE; 256],
            count: 0,
        }
    }

    pub fn push(&mut self, mv: Move) {
        if self.count < 256 {
            self.moves[self.count] = mv;
            self.count += 1;
        }
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn swap(&mut self, i: usize, j: usize) {
        self.moves.swap(i, j);
    }

    pub fn as_slice(&self) -> &[Move] {
        &self.moves[0..self.count]
    }

    pub fn as_mut_slice(&mut self) -> &mut [Move] {
        &mut self.moves[0..self.count]
    }
}

impl std::ops::Index<usize> for MoveList {
    type Output = Move;
    fn index(&self, index: usize) -> &Self::Output {
        &self.moves[index]
    }
}

pub struct MoveListIterator<'a> {
    move_list: &'a MoveList,
    index: usize,
}

impl<'a> Iterator for MoveListIterator<'a> {
    type Item = Move;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.move_list.count {
            let mv = self.move_list.moves[self.index];
            self.index += 1;
            Some(mv)
        } else {
            None
        }
    }
}

impl<'a> IntoIterator for &'a MoveList {
    type Item = Move;
    type IntoIter = MoveListIterator<'a>;
    fn into_iter(self) -> Self::IntoIter {
        MoveListIterator {
            move_list: self,
            index: 0,
        }
    }
}

pub fn generate_moves(pos: &Position) -> MoveList {
    let mut moves = MoveList::new();
    generate_pseudo_legal_moves(pos, &mut moves, false);

    // Filter legal moves
    let mut legal_moves = MoveList::new();
    let us = pos.side_to_move();
    for i in 0..moves.len() {
        let mv = moves[i];
        let mut next_pos = pos.clone();
        next_pos.make_move(mv);
        if !next_pos.is_in_check(us) {
            legal_moves.push(mv);
        }
    }
    legal_moves
}

pub fn generate_captures(pos: &Position) -> MoveList {
    let mut moves = MoveList::new();
    generate_pseudo_legal_moves(pos, &mut moves, true);

    // Filter legal moves
    let mut legal_moves = MoveList::new();
    let us = pos.side_to_move();
    for i in 0..moves.len() {
        let mv = moves[i];
        let mut next_pos = pos.clone();
        next_pos.make_move(mv);
        if !next_pos.is_in_check(us) {
            legal_moves.push(mv);
        }
    }
    legal_moves
}

fn generate_pseudo_legal_moves(pos: &Position, moves: &mut MoveList, captures_only: bool) {
    let us = pos.side_to_move();
    let them = us.opposite();
    let us_occ = pos.occupancy_color(us);
    let them_occ = pos.occupancy_color(them);
    let occ = us_occ | them_occ;

    // For move generation, we should not capture the enemy king.
    // In fact, if we can capture the king, the previous move was illegal.
    let target_mask = !us_occ & !pos.pieces(them, Piece::King);
    let capture_mask = them_occ & !pos.pieces(them, Piece::King);

    // Pawns
    let mut pawns = pos.pieces(us, Piece::Pawn);
    while !pawns.is_empty() {
        let from = pawns.pop_lsb();
        generate_pawn_moves(pos, from, moves, captures_only);
    }

    // Knights
    let mut knights = pos.pieces(us, Piece::Knight);
    while !knights.is_empty() {
        let from = knights.pop_lsb();
        let mut attacks = magics::knight_attacks(from) & target_mask;
        if captures_only {
            attacks &= capture_mask;
        }
        while !attacks.is_empty() {
            moves.push(Move::new(from, attacks.pop_lsb(), 0, 0));
        }
    }

    // Bishops
    let mut bishops = pos.pieces(us, Piece::Bishop);
    while !bishops.is_empty() {
        let from = bishops.pop_lsb();
        let mut attacks = magics::bishop_attacks(from, occ) & target_mask;
        if captures_only {
            attacks &= capture_mask;
        }
        while !attacks.is_empty() {
            moves.push(Move::new(from, attacks.pop_lsb(), 0, 0));
        }
    }

    // Rooks
    let mut rooks = pos.pieces(us, Piece::Rook);
    while !rooks.is_empty() {
        let from = rooks.pop_lsb();
        let mut attacks = magics::rook_attacks(from, occ) & target_mask;
        if captures_only {
            attacks &= capture_mask;
        }
        while !attacks.is_empty() {
            moves.push(Move::new(from, attacks.pop_lsb(), 0, 0));
        }
    }

    // Queens
    let mut queens = pos.pieces(us, Piece::Queen);
    while !queens.is_empty() {
        let from = queens.pop_lsb();
        let mut attacks = magics::queen_attacks(from, occ) & target_mask;
        if captures_only {
            attacks &= capture_mask;
        }
        while !attacks.is_empty() {
            moves.push(Move::new(from, attacks.pop_lsb(), 0, 0));
        }
    }

    // King
    let king_bb = pos.pieces(us, Piece::King);
    if !king_bb.is_empty() {
        let from = king_bb.lsb();
        let mut attacks = magics::king_attacks(from) & target_mask;
        if captures_only {
            attacks &= capture_mask;
        }
        while !attacks.is_empty() {
            moves.push(Move::new(from, attacks.pop_lsb(), 0, 0));
        }
    }

    // Castling (only if not captures_only)
    if !captures_only {
        generate_castling_moves(pos, moves);
    }
}

fn generate_pawn_moves(pos: &Position, from: Square, moves: &mut MoveList, captures_only: bool) {
    let us = pos.side_to_move();
    let them = us.opposite();
    let occ = pos.occupancy();
    let them_occ = pos.occupancy_color(them);

    let rank = from.rank();

    if us == Color::White {
        // Single push
        if !captures_only {
            let to = Square(from.0 + 8);
            if !occ.is_set(to) {
                if rank == 6 {
                    // Promotion
                    push_promotions(from, to, moves);
                } else {
                    moves.push(Move::new(from, to, 0, 0));
                    // Double push
                    if rank == 1 {
                        let to2 = Square(from.0 + 16);
                        if !occ.is_set(to2) {
                            moves.push(Move::new(from, to2, 0, 0));
                        }
                    }
                }
            }
        }
        // Captures
        let mut attacks = magics::pawn_attacks(Color::White, from) & them_occ;
        attacks &= !pos.pieces(Color::Black, Piece::King); // Don't capture king
        while !attacks.is_empty() {
            let to = attacks.pop_lsb();
            if rank == 6 {
                push_promotions(from, to, moves);
            } else {
                moves.push(Move::new(from, to, 0, 0));
            }
        }
        // En passant
        if let Some(ep_sq) = pos.en_passant_square() {
            if magics::pawn_attacks(Color::White, from).is_set(ep_sq) {
                moves.push(Move::new(from, ep_sq, 2, 0));
            }
        }
    } else {
        // Black
        if !captures_only {
            let to = Square(from.0 - 8);
            if !occ.is_set(to) {
                if rank == 1 {
                    // Promotion
                    push_promotions(from, to, moves);
                } else {
                    moves.push(Move::new(from, to, 0, 0));
                    // Double push
                    if rank == 6 {
                        let to2 = Square(from.0 - 16);
                        if !occ.is_set(to2) {
                            moves.push(Move::new(from, to2, 0, 0));
                        }
                    }
                }
            }
        }
        // Captures
        let mut attacks = magics::pawn_attacks(Color::Black, from) & them_occ;
        attacks &= !pos.pieces(Color::White, Piece::King); // Don't capture king
        while !attacks.is_empty() {
            let to = attacks.pop_lsb();
            if rank == 1 {
                push_promotions(from, to, moves);
            } else {
                moves.push(Move::new(from, to, 0, 0));
            }
        }
        // En passant
        if let Some(ep_sq) = pos.en_passant_square() {
            if magics::pawn_attacks(Color::Black, from).is_set(ep_sq) {
                moves.push(Move::new(from, ep_sq, 2, 0));
            }
        }
    }
}

fn push_promotions(from: Square, to: Square, moves: &mut MoveList) {
    for promo in 0..4 {
        moves.push(Move::new(from, to, 1, promo));
    }
}

fn generate_castling_moves(pos: &Position, moves: &mut MoveList) {
    let us = pos.side_to_move();
    let rights = pos.castling_rights();
    let occ = pos.occupancy();

    if us == Color::White {
        if rights.has(CastlingRights::WK)
            && !occ.is_set(Square(5))
            && !occ.is_set(Square(6))
            && !is_square_attacked(pos, Square(4), Color::Black)
            && !is_square_attacked(pos, Square(5), Color::Black)
            && !is_square_attacked(pos, Square(6), Color::Black)
        {
            moves.push(Move::new(Square(4), Square(6), 3, 0));
        }
        if rights.has(CastlingRights::WQ)
            && !occ.is_set(Square(1))
            && !occ.is_set(Square(2))
            && !occ.is_set(Square(3))
            && !is_square_attacked(pos, Square(4), Color::Black)
            && !is_square_attacked(pos, Square(3), Color::Black)
            && !is_square_attacked(pos, Square(2), Color::Black)
        {
            moves.push(Move::new(Square(4), Square(2), 3, 0));
        }
    } else {
        if rights.has(CastlingRights::BK)
            && !occ.is_set(Square(61))
            && !occ.is_set(Square(62))
            && !is_square_attacked(pos, Square(60), Color::White)
            && !is_square_attacked(pos, Square(61), Color::White)
            && !is_square_attacked(pos, Square(62), Color::White)
        {
            moves.push(Move::new(Square(60), Square(62), 3, 0));
        }
        if rights.has(CastlingRights::BQ)
            && !occ.is_set(Square(57))
            && !occ.is_set(Square(58))
            && !occ.is_set(Square(59))
            && !is_square_attacked(pos, Square(60), Color::White)
            && !is_square_attacked(pos, Square(59), Color::White)
            && !is_square_attacked(pos, Square(58), Color::White)
        {
            moves.push(Move::new(Square(60), Square(58), 3, 0));
        }
    }
}

pub fn is_square_attacked(pos: &Position, sq: Square, by_color: Color) -> bool {
    let occ = pos.occupancy();

    // Pawns
    if !(magics::pawn_attacks(by_color.opposite(), sq) & pos.pieces(by_color, Piece::Pawn))
        .is_empty()
    {
        return true;
    }

    // Knights
    if !(magics::knight_attacks(sq) & pos.pieces(by_color, Piece::Knight)).is_empty() {
        return true;
    }

    // King
    if !(magics::king_attacks(sq) & pos.pieces(by_color, Piece::King)).is_empty() {
        return true;
    }

    // Bishops / Queens
    let diag_attackers = pos.pieces(by_color, Piece::Bishop) | pos.pieces(by_color, Piece::Queen);
    if !(magics::bishop_attacks(sq, occ) & diag_attackers).is_empty() {
        return true;
    }

    // Rooks / Queens
    let straight_attackers = pos.pieces(by_color, Piece::Rook) | pos.pieces(by_color, Piece::Queen);
    if !(magics::rook_attacks(sq, occ) & straight_attackers).is_empty() {
        return true;
    }

    false
}

pub fn perft(pos: &Position, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }

    let moves = generate_moves(pos);
    if depth == 1 {
        return moves.len() as u64;
    }

    let mut nodes = 0;
    for i in 0..moves.len() {
        let mut next_pos = pos.clone();
        next_pos.make_move(moves[i]);
        nodes += perft(&next_pos, depth - 1);
    }
    nodes
}
