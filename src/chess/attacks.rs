use crate::chess::{
    bitboard::Bitboard, castling_rights::CastlingRights, color::Color, coords::Square,
    hyperbola::HyperbolaQsc, piece::Piece, piecetype::PieceType, position::Position, r#move::Move,
};

use arrayvec::ArrayVec;

const HYPERBOLA: HyperbolaQsc = HyperbolaQsc::new();
const PROMOTION_PIECES: [PieceType; 4] = [
    PieceType::Queen,
    PieceType::Rook,
    PieceType::Bishop,
    PieceType::Knight,
];

#[inline(always)]
fn pop_lsb(bb: &mut u64) -> u32 {
    let idx = bb.trailing_zeros() as u32;
    *bb &= *bb - 1;
    idx
}

/// Return every pseudo-legal move for the current position.
pub fn pseudo_legal_moves(pos: &Position) -> ArrayVec<Move, 256> {
    let mut moves = ArrayVec::new();
    let side = pos.side_to_move();
    let occupancy_bits = pos.occupied().bits();
    let occupancy = Bitboard::new(occupancy_bits);
    let ep_square = pos.ep_square();

    let mut pawns = pos.pieces_bb_color(side, PieceType::Pawn).bits();
    while pawns != 0 {
        let from_idx = pop_lsb(&mut pawns);
        let from_sq = Square::new(from_idx);
        let from_rank = from_idx / 8;
        let direction: i32 = if side == Color::White { 8 } else { -8 };
        let one_step = from_idx as i32 + direction;

        if one_step >= 0 && one_step < 64 {
            let to_sq = Square::new(one_step as u32);
            if pos.piece_at(to_sq) == Piece::none() {
                if (side == Color::White && one_step >= 56)
                    || (side == Color::Black && one_step < 8)
                {
                    for &promo in PROMOTION_PIECES.iter() {
                        moves.push(Move::promotion(from_sq, to_sq, Piece::new(promo, side)));
                    }
                } else {
                    moves.push(Move::normal(from_sq, to_sq));

                    let start_rank = if side == Color::White { 1 } else { 6 };
                    if from_rank == start_rank {
                        let two_step = from_idx as i32 + 2 * direction;
                        if two_step >= 0 && two_step < 64 {
                            let mid_sq = Square::new(one_step as u32);
                            let dbl_sq = Square::new(two_step as u32);
                            if pos.piece_at(mid_sq) == Piece::none()
                                && pos.piece_at(dbl_sq) == Piece::none()
                            {
                                moves.push(Move::normal(from_sq, dbl_sq));
                            }
                        }
                    }
                }
            }
        }

        let mut attacks = pawn(side, from_sq).bits();
        while attacks != 0 {
            let to_idx = pop_lsb(&mut attacks);
            let to_sq = Square::new(to_idx);

            if ep_square != Square::NONE && to_sq == ep_square {
                moves.push(Move::en_passant(from_sq, to_sq));
                continue;
            }

            let target_piece = pos.piece_at(to_sq);
            if target_piece != Piece::none() && target_piece.color() != side {
                if (side == Color::White && to_idx >= 56) || (side == Color::Black && to_idx < 8) {
                    for &promo in PROMOTION_PIECES.iter() {
                        moves.push(Move::promotion(from_sq, to_sq, Piece::new(promo, side)));
                    }
                } else {
                    moves.push(Move::normal(from_sq, to_sq));
                }
            }
        }
    }

    let mut knights = pos.pieces_bb_color(side, PieceType::Knight).bits();
    while knights != 0 {
        let from_idx = pop_lsb(&mut knights);
        let from_sq = Square::new(from_idx);
        let mut targets = knight(from_sq).bits();
        while targets != 0 {
            let to_idx = pop_lsb(&mut targets);
            let to_sq = Square::new(to_idx);
            let target_piece = pos.piece_at(to_sq);
            if target_piece == Piece::none() || target_piece.color() != side {
                moves.push(Move::normal(from_sq, to_sq));
            }
        }
    }

    let mut bishops = pos.pieces_bb_color(side, PieceType::Bishop).bits();
    while bishops != 0 {
        let from_idx = pop_lsb(&mut bishops);
        let from_sq = Square::new(from_idx);
        let mut targets = bishop(from_sq, occupancy).bits();
        while targets != 0 {
            let to_idx = pop_lsb(&mut targets);
            let to_sq = Square::new(to_idx);
            let target_piece = pos.piece_at(to_sq);
            if target_piece == Piece::none() || target_piece.color() != side {
                moves.push(Move::normal(from_sq, to_sq));
            }
        }
    }

    let mut rooks = pos.pieces_bb_color(side, PieceType::Rook).bits();
    while rooks != 0 {
        let from_idx = pop_lsb(&mut rooks);
        let from_sq = Square::new(from_idx);
        let mut targets = rook(from_sq, occupancy).bits();
        while targets != 0 {
            let to_idx = pop_lsb(&mut targets);
            let to_sq = Square::new(to_idx);
            let target_piece = pos.piece_at(to_sq);
            if target_piece == Piece::none() || target_piece.color() != side {
                moves.push(Move::normal(from_sq, to_sq));
            }
        }
    }

    let mut queens = pos.pieces_bb_color(side, PieceType::Queen).bits();
    while queens != 0 {
        let from_idx = pop_lsb(&mut queens);
        let from_sq = Square::new(from_idx);
        let mut targets = queen(from_sq, occupancy).bits();
        while targets != 0 {
            let to_idx = pop_lsb(&mut targets);
            let to_sq = Square::new(to_idx);
            let target_piece = pos.piece_at(to_sq);
            if target_piece == Piece::none() || target_piece.color() != side {
                moves.push(Move::normal(from_sq, to_sq));
            }
        }
    }

    let mut kings = pos.pieces_bb_color(side, PieceType::King).bits();
    while kings != 0 {
        let from_idx = pop_lsb(&mut kings);
        let from_sq = Square::new(from_idx);
        let mut targets = king(from_sq).bits();
        while targets != 0 {
            let to_idx = pop_lsb(&mut targets);
            let to_sq = Square::new(to_idx);
            let target_piece = pos.piece_at(to_sq);
            if target_piece == Piece::none() || target_piece.color() != side {
                moves.push(Move::normal(from_sq, to_sq));
            }
        }
    }

    // Castling moves, king captures rook

    let king_sq = pos.king_sq(side);
    let castling_rights = pos.castling_rights();

    // check in check
    if super_attacks_from_square(king_sq, side, pos).bits() != 0 {
        return moves;
    }

    if side == Color::White {
        // White kingside castling
        if castling_rights.contains(CastlingRights::WHITE_KING_SIDE) {
            let f1 = Square::F1;
            let g1 = Square::G1;
            if super_attacks_from_square(f1, side, pos).bits() == 0
                && super_attacks_from_square(g1, side, pos).bits() == 0
            {
                if pos.piece_at(f1) == Piece::none() && pos.piece_at(g1) == Piece::none() {
                    moves.push(Move::castle(king_sq, Square::H1));
                }
            }
        }

        // White queenside castling
        if castling_rights.contains(CastlingRights::WHITE_QUEEN_SIDE) {
            let b1 = Square::B1;
            let c1 = Square::C1;
            let d1 = Square::D1;
            if super_attacks_from_square(c1, side, pos).bits() == 0
                && super_attacks_from_square(d1, side, pos).bits() == 0
            {
                if pos.piece_at(b1) == Piece::none()
                    && pos.piece_at(c1) == Piece::none()
                    && pos.piece_at(d1) == Piece::none()
                {
                    moves.push(Move::castle(king_sq, Square::A1));
                }
            }
        }
    } else {
        // Black kingside castling
        if castling_rights.contains(CastlingRights::BLACK_KING_SIDE) {
            let f8 = Square::F8;
            let g8 = Square::G8;
            if super_attacks_from_square(f8, side, pos).bits() == 0
                && super_attacks_from_square(g8, side, pos).bits() == 0
            {
                if pos.piece_at(f8) == Piece::none() && pos.piece_at(g8) == Piece::none() {
                    moves.push(Move::castle(king_sq, Square::H8));
                }
            }
        }

        // Black queenside castling
        if castling_rights.contains(CastlingRights::BLACK_QUEEN_SIDE) {
            let b8 = Square::B8;
            let c8 = Square::C8;
            let d8 = Square::D8;
            if super_attacks_from_square(c8, side, pos).bits() == 0
                && super_attacks_from_square(d8, side, pos).bits() == 0
            {
                if pos.piece_at(b8) == Piece::none()
                    && pos.piece_at(c8) == Piece::none()
                    && pos.piece_at(d8) == Piece::none()
                {
                    moves.push(Move::castle(king_sq, Square::A8));
                }
            }
        }
    }

    moves
}

fn super_attacks_from_square(sq: Square, c: Color, pos: &Position) -> Bitboard {
    Bitboard::from_u64(
        pawn(c, sq).bits() & pos.pieces_bb_color(!c, PieceType::Pawn).bits()
            | knight(sq).bits() & pos.pieces_bb_color(!c, PieceType::Knight).bits()
            | bishop(sq, pos.occupied()).bits()
                & (pos.pieces_bb_color(!c, PieceType::Bishop).bits()
                    | pos.pieces_bb_color(!c, PieceType::Queen).bits())
            | rook(sq, pos.occupied()).bits()
                & (pos.pieces_bb_color(!c, PieceType::Rook).bits()
                    | pos.pieces_bb_color(!c, PieceType::Queen).bits())
            | king(sq).bits() & pos.pieces_bb_color(!c, PieceType::King).bits(),
    )
}

/// Get pseudo pawn attacks for a given color and square.
pub fn pawn(color: Color, sq: Square) -> Bitboard {
    Bitboard::new(PAWN_ATTACKS[color as usize][sq.index() as usize])
}

/// Get pseudo knight attacks for a given square.
pub fn knight(sq: Square) -> Bitboard {
    Bitboard::new(KNIGHT_ATTACKS[sq.index() as usize])
}

/// Get pseudo bishop attacks for a given square and occupied squares.
pub fn bishop(sq: Square, occupied: Bitboard) -> Bitboard {
    HYPERBOLA.bishop_attack(sq, occupied)
}

/// Get pseudo rook attacks for a given square and occupied squares.
pub fn rook(sq: Square, occupied: Bitboard) -> Bitboard {
    HYPERBOLA.rook_attack(sq, occupied)
}

/// Get pseudo queen attacks for a given square and occupied squares.
pub fn queen(sq: Square, occupied: Bitboard) -> Bitboard {
    Bitboard::from_u64(bishop(sq, occupied).bits() | rook(sq, occupied).bits())
}

/// Get pseudo king attacks for a given square.
pub fn king(sq: Square) -> Bitboard {
    Bitboard::new(KING_ATTACKS[sq.index() as usize])
}

/// Get pseudo attacks for a given piece type, square, and occupied squares.
pub fn piece_attacks(pt: PieceType, sq: Square, occupied: Bitboard) -> Bitboard {
    match pt {
        PieceType::Knight => knight(sq),
        PieceType::Bishop => bishop(sq, occupied),
        PieceType::Rook => rook(sq, occupied),
        PieceType::Queen => queen(sq, occupied),
        PieceType::King => king(sq),
        _ => panic!("Invalid piece type"),
    }
}

static PAWN_ATTACKS: [[u64; 64]; 2] = [
    // White
    [
        0x200,
        0x500,
        0xa00,
        0x1400,
        0x2800,
        0x5000,
        0xa000,
        0x4000,
        0x20000,
        0x50000,
        0xa0000,
        0x140000,
        0x280000,
        0x500000,
        0xa00000,
        0x400000,
        0x2000000,
        0x5000000,
        0xa000000,
        0x14000000,
        0x28000000,
        0x50000000,
        0xa0000000,
        0x40000000,
        0x200000000,
        0x500000000,
        0xa00000000,
        0x1400000000,
        0x2800000000,
        0x5000000000,
        0xa000000000,
        0x4000000000,
        0x20000000000,
        0x50000000000,
        0xa0000000000,
        0x140000000000,
        0x280000000000,
        0x500000000000,
        0xa00000000000,
        0x400000000000,
        0x2000000000000,
        0x5000000000000,
        0xa000000000000,
        0x14000000000000,
        0x28000000000000,
        0x50000000000000,
        0xa0000000000000,
        0x40000000000000,
        0x200000000000000,
        0x500000000000000,
        0xa00000000000000,
        0x1400000000000000,
        0x2800000000000000,
        0x5000000000000000,
        0xa000000000000000,
        0x4000000000000000,
        0x0,
        0x0,
        0x0,
        0x0,
        0x0,
        0x0,
        0x0,
        0x0,
    ],
    // Black
    [
        0x0,
        0x0,
        0x0,
        0x0,
        0x0,
        0x0,
        0x0,
        0x0,
        0x2,
        0x5,
        0xa,
        0x14,
        0x28,
        0x50,
        0xa0,
        0x40,
        0x200,
        0x500,
        0xa00,
        0x1400,
        0x2800,
        0x5000,
        0xa000,
        0x4000,
        0x20000,
        0x50000,
        0xa0000,
        0x140000,
        0x280000,
        0x500000,
        0xa00000,
        0x400000,
        0x2000000,
        0x5000000,
        0xa000000,
        0x14000000,
        0x28000000,
        0x50000000,
        0xa0000000,
        0x40000000,
        0x200000000,
        0x500000000,
        0xa00000000,
        0x1400000000,
        0x2800000000,
        0x5000000000,
        0xa000000000,
        0x4000000000,
        0x20000000000,
        0x50000000000,
        0xa0000000000,
        0x140000000000,
        0x280000000000,
        0x500000000000,
        0xa00000000000,
        0x400000000000,
        0x2000000000000,
        0x5000000000000,
        0xa000000000000,
        0x14000000000000,
        0x28000000000000,
        0x50000000000000,
        0xa0000000000000,
        0x40000000000000,
    ],
];

static KNIGHT_ATTACKS: [u64; 64] = [
    0x0000000000020400,
    0x0000000000050800,
    0x00000000000A1100,
    0x0000000000142200,
    0x0000000000284400,
    0x0000000000508800,
    0x0000000000A01000,
    0x0000000000402000,
    0x0000000002040004,
    0x0000000005080008,
    0x000000000A110011,
    0x0000000014220022,
    0x0000000028440044,
    0x0000000050880088,
    0x00000000A0100010,
    0x0000000040200020,
    0x0000000204000402,
    0x0000000508000805,
    0x0000000A1100110A,
    0x0000001422002214,
    0x0000002844004428,
    0x0000005088008850,
    0x000000A0100010A0,
    0x0000004020002040,
    0x0000020400040200,
    0x0000050800080500,
    0x00000A1100110A00,
    0x0000142200221400,
    0x0000284400442800,
    0x0000508800885000,
    0x0000A0100010A000,
    0x0000402000204000,
    0x0002040004020000,
    0x0005080008050000,
    0x000A1100110A0000,
    0x0014220022140000,
    0x0028440044280000,
    0x0050880088500000,
    0x00A0100010A00000,
    0x0040200020400000,
    0x0204000402000000,
    0x0508000805000000,
    0x0A1100110A000000,
    0x1422002214000000,
    0x2844004428000000,
    0x5088008850000000,
    0xA0100010A0000000,
    0x4020002040000000,
    0x0400040200000000,
    0x0800080500000000,
    0x1100110A00000000,
    0x2200221400000000,
    0x4400442800000000,
    0x8800885000000000,
    0x100010A000000000,
    0x2000204000000000,
    0x0004020000000000,
    0x0008050000000000,
    0x00110A0000000000,
    0x0022140000000000,
    0x0044280000000000,
    0x0088500000000000,
    0x0010A00000000000,
    0x0020400000000000,
];

static KING_ATTACKS: [u64; 64] = [
    0x0000000000000302,
    0x0000000000000705,
    0x0000000000000E0A,
    0x0000000000001C14,
    0x0000000000003828,
    0x0000000000007050,
    0x000000000000E0A0,
    0x000000000000C040,
    0x0000000000030203,
    0x0000000000070507,
    0x00000000000E0A0E,
    0x00000000001C141C,
    0x0000000000382838,
    0x0000000000705070,
    0x0000000000E0A0E0,
    0x0000000000C040C0,
    0x0000000003020300,
    0x0000000007050700,
    0x000000000E0A0E00,
    0x000000001C141C00,
    0x0000000038283800,
    0x0000000070507000,
    0x00000000E0A0E000,
    0x00000000C040C000,
    0x0000000302030000,
    0x0000000705070000,
    0x0000000E0A0E0000,
    0x0000001C141C0000,
    0x0000003828380000,
    0x0000007050700000,
    0x000000E0A0E00000,
    0x000000C040C00000,
    0x0000030203000000,
    0x0000070507000000,
    0x00000E0A0E000000,
    0x00001C141C000000,
    0x0000382838000000,
    0x0000705070000000,
    0x0000E0A0E0000000,
    0x0000C040C0000000,
    0x0003020300000000,
    0x0007050700000000,
    0x000E0A0E00000000,
    0x001C141C00000000,
    0x0038283800000000,
    0x0070507000000000,
    0x00E0A0E000000000,
    0x00C040C000000000,
    0x0302030000000000,
    0x0705070000000000,
    0x0E0A0E0000000000,
    0x1C141C0000000000,
    0x3828380000000000,
    0x7050700000000000,
    0xE0A0E00000000000,
    0xC040C00000000000,
    0x0203000000000000,
    0x0507000000000000,
    0x0A0E000000000000,
    0x141C000000000000,
    0x2838000000000000,
    0x5070000000000000,
    0xA0E0000000000000,
    0x40C0000000000000,
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chess::{piecetype::PieceType, position::Position, r#move::MoveType};

    const STARTPOS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    fn perft(pos: &Position, depth: u32) -> u64 {
        if depth == 0 {
            return 1;
        }

        let mut nodes = 0;

        let moves = pseudo_legal_moves(&pos);

        for mv in moves {
            let new_pos = pos.after_move(mv);
            if !new_pos.is_checked(pos.side_to_move()) {
                nodes += perft(&new_pos, depth - 1);
            }
        }

        nodes
    }

    fn split_perft(pos: &Position, depth: u32) -> u64 {
        let moves = pseudo_legal_moves(&pos);
        let mut total_nodes = 0;

        for mv in moves {
            let new_pos = pos.after_move(mv);
            if !new_pos.is_checked(pos.side_to_move()) {
                let nodes = perft(&new_pos, depth - 1);
                total_nodes += nodes;
                println!("{}: {}", mv.as_uci(), nodes);
            }
        }

        println!("Total nodes: {}", total_nodes);
        total_nodes
    }

    #[test]
    fn test_bishop_mask() {
        assert_eq!(
            bishop(Square::new(27), Bitboard::new(0)).bits(),
            9241705379636978241
        );
        assert_eq!(
            rook(Square::new(27), Bitboard::new(0)).bits(),
            578721386714368008
        );
    }

    #[test]
    fn test_pseudo_moves_startpos() {
        let pos = &Position::from_fen(STARTPOS).unwrap();
        let moves = pseudo_legal_moves(&pos);
        assert_eq!(moves.len(), 20);
    }

    #[test]
    fn test_knight_pseudo_moves() {
        let pos = &Position::from_fen("k7/8/8/3N4/8/8/8/6K1 w - - 0 1").unwrap();
        let moves = pseudo_legal_moves(&pos);
        let knight_moves = moves
            .iter()
            .filter(|m| pos.piece_at(m.from()).piece_type() == PieceType::Knight)
            .count();
        assert_eq!(knight_moves, 8);
    }

    #[test]
    fn test_en_passant_included() {
        let pos = &Position::from_fen("k7/8/8/3pP3/8/8/8/6K1 w - d6 0 1").unwrap();
        let moves = pseudo_legal_moves(&pos);
        assert!(moves.iter().any(|m| m.mtype() == MoveType::EnPassant));
    }

    #[test]
    fn test_perft_startpos_depth_1() {
        let pos = &Position::from_fen(STARTPOS).unwrap();
        assert_eq!(split_perft(pos, 1), 20);
    }

    #[test]
    fn test_perft_startpos_depth_2() {
        assert_eq!(split_perft(&Position::from_fen(STARTPOS).unwrap(), 2), 400);
    }

    #[test]
    fn test_perft_startpos_depth_3() {
        assert_eq!(split_perft(&Position::from_fen(STARTPOS).unwrap(), 3), 8902);
    }

    #[test]
    fn test_perft_startpos_depth_4() {
        assert_eq!(
            split_perft(&Position::from_fen(STARTPOS).unwrap(), 4),
            197281
        );
    }

    #[test]
    fn test_perft_startpos_depth_5() {
        assert_eq!(
            split_perft(&Position::from_fen(STARTPOS).unwrap(), 5),
            4865609
        );
    }

    #[test]
    fn test_perft_startpos_depth_7() {
        assert_eq!(
            split_perft(
                &Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
                    .unwrap(),
                7
            ),
            3195901860
        );
    }

    #[test]
    fn test_perft_custom_position_1() {
        assert_eq!(
            split_perft(
                &Position::from_fen("rnbqkbnr/ppp1pppp/3p4/8/8/2P5/PP1PPPPP/RNBQKBNR w KQkq - 0 2")
                    .unwrap(),
                1
            ),
            21
        );
    }

    #[test]
    fn test_perft_custom_position_2() {
        assert_eq!(
            split_perft(
                &Position::from_fen("rnbqkbnr/pppppppp/8/8/8/2P5/PP1PPPPP/RNBQKBNR b KQkq - 0 1")
                    .unwrap(),
                2
            ),
            420
        );
    }

    #[test]
    fn test_perft_castle_position() {
        assert_eq!(
            split_perft(
                &Position::from_fen(
                    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"
                )
                .unwrap(),
                1
            ),
            48
        );
    }

    #[test]
    fn test_perft_complex_position_1() {
        assert_eq!(
            split_perft(
                &Position::from_fen(
                    "r3k2r/p1ppqpb1/bnN1pnp1/3P4/1p2P3/2N2Q1p/PPPBBPPP/R3K2R b KQkq - 1 1"
                )
                .unwrap(),
                1
            ),
            41
        );
    }

    #[test]
    fn test_perft_complex_position_2() {
        assert_eq!(
            split_perft(
                &Position::from_fen(
                    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/P1N2Q2/1PPBBPpP/R3K2R w KQkq - 0 2"
                )
                .unwrap(),
                1
            ),
            48
        );
    }

    #[test]
    fn test_perft_complex_position_3() {
        assert_eq!(
            split_perft(
                &Position::from_fen(
                    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"
                )
                .unwrap(),
                2
            ),
            2039
        );
    }

    #[test]
    fn test_perft_complex_position_4() {
        assert_eq!(
            split_perft(
                &Position::from_fen(
                    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/P1N2Q1p/1PPBBPPP/R3K2R b KQkq - 0 1"
                )
                .unwrap(),
                2
            ),
            2186
        );
    }

    #[test]
    fn test_perft_complex_position_5() {
        assert_eq!(
            split_perft(
                &Position::from_fen(
                    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"
                )
                .unwrap(),
                3
            ),
            97862
        );
    }

    #[test]
    fn test_perft_complex_position_6() {
        assert_eq!(
            split_perft(
                &Position::from_fen(
                    "r3k2r/p1p1qpb1/bn1ppnp1/1B1PN3/1p2P3/P1N2Q1p/1PPB1PPP/R3K2R b KQkq - 1 2"
                )
                .unwrap(),
                1
            ),
            7
        );
    }

    #[test]
    fn test_perft_complex_position_7() {
        assert_eq!(
            split_perft(
                &Position::from_fen(
                    "r3k2r/p1p1qpb1/bn1ppnp1/3PN3/1p2P3/P1N2Q1p/1PPBBPPP/R3K2R w KQkq - 0 2"
                )
                .unwrap(),
                2
            ),
            2135
        );
    }

    #[test]
    fn test_perft_complex_position_8() {
        assert_eq!(
            split_perft(
                &Position::from_fen(
                    "r3k2r/p1ppqpb1/bn2p1p1/3PN3/1p2n3/P1N2Q1p/1PPBBPPP/R3K2R w KQkq - 0 2"
                )
                .unwrap(),
                2
            ),
            2717
        );
    }

    #[test]
    fn test_perft_complex_position_9() {
        assert_eq!(
            split_perft(
                &Position::from_fen(
                    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/P1N2Q1p/1PPBBPPP/R3K2R b KQkq - 0 1"
                )
                .unwrap(),
                3
            ),
            94405
        );
    }

    #[test]
    fn test_perft_complex_position_10() {
        assert_eq!(
            split_perft(
                &Position::from_fen(
                    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"
                )
                .unwrap(),
                4
            ),
            4085603
        );
    }

    #[test]
    fn test_perft_complex_position_11() {
        assert_eq!(
            split_perft(
                &Position::from_fen(
                    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"
                )
                .unwrap(),
                5
            ),
            193690690
        );
    }

    #[test]
    fn test_perft_endgame_position() {
        assert_eq!(
            split_perft(
                &Position::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap(),
                7
            ),
            178633661
        );
    }

    #[test]
    fn test_perft_tactical_position_1() {
        assert_eq!(
            split_perft(
                &&Position::from_fen(
                    "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1"
                )
                .unwrap(),
                6
            ),
            706045033
        );
    }

    #[test]
    fn test_perft_tactical_position_2() {
        assert_eq!(
            split_perft(
                &Position::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8")
                    .unwrap(),
                5
            ),
            89941194
        );
    }

    #[test]
    fn test_perft_tactical_position_3() {
        assert_eq!(
            split_perft(
                &Position::from_fen(
                    "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 1"
                )
                .unwrap(),
                5
            ),
            164075551
        );
    }
}
