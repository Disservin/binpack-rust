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
fn pop_lsb(bb: &mut u64) -> Square {
    let idx = bb.trailing_zeros();
    *bb &= *bb - 1;
    Square::new(idx)
}

/// Return every pseudo-legal move for the current position.
pub fn pseudo_legal_moves(pos: &Position) -> ArrayVec<Move, 256> {
    let mut moves = ArrayVec::new();
    let side = pos.side_to_move();
    let occupancy = Bitboard::new(pos.occupied().bits());

    generate_pawn_moves(pos, side, &mut moves);
    generate_piece_moves::<Knight>(pos, side, occupancy, &mut moves);
    generate_piece_moves::<Bishop>(pos, side, occupancy, &mut moves);
    generate_piece_moves::<Rook>(pos, side, occupancy, &mut moves);
    generate_piece_moves::<Queen>(pos, side, occupancy, &mut moves);
    generate_piece_moves::<King>(pos, side, occupancy, &mut moves);
    generate_castling_moves(pos, side, &mut moves);

    moves
}

fn generate_pawn_moves(pos: &Position, side: Color, moves: &mut ArrayVec<Move, 256>) {
    let mut pawns = pos.pieces_bb_color(side, PieceType::Pawn).bits();
    let direction = if side == Color::White { 8 } else { -8 };
    let promotion_rank_start = if side == Color::White { 56 } else { 0 };
    let promotion_rank_end = if side == Color::White { 64 } else { 8 };

    while pawns != 0 {
        let from_sq = pop_lsb(&mut pawns);

        generate_pawn_pushes(
            pos,
            side,
            from_sq,
            direction,
            promotion_rank_start,
            promotion_rank_end,
            moves,
        );

        generate_pawn_captures(
            pos,
            side,
            from_sq,
            promotion_rank_start,
            promotion_rank_end,
            moves,
        );
    }
}

fn generate_pawn_pushes(
    pos: &Position,
    side: Color,
    from_sq: Square,
    direction: i32,
    promotion_start: i32,
    promotion_end: i32,
    moves: &mut ArrayVec<Move, 256>,
) {
    let start_rank = if side == Color::White { 1 } else { 6 };

    let one_step = from_sq.index() as i32 + direction;
    if !(0..64).contains(&one_step) || pos.piece_at(Square::new(one_step as u32)) != Piece::none() {
        return;
    }

    let to_sq = Square::new(one_step as u32);

    if (promotion_start..promotion_end).contains(&one_step) {
        add_promotions(from_sq, to_sq, side, moves);
    } else {
        moves.push(Move::normal(from_sq, to_sq));

        // Double push
        if from_sq.index() / 8 == start_rank {
            let two_step = one_step + direction;
            if (0..64).contains(&two_step)
                && pos.piece_at(Square::new(two_step as u32)) == Piece::none()
            {
                moves.push(Move::normal(from_sq, Square::new(two_step as u32)));
            }
        }
    }
}

fn generate_pawn_captures(
    pos: &Position,
    side: Color,
    from_sq: Square,
    promotion_start: i32,
    promotion_end: i32,
    moves: &mut ArrayVec<Move, 256>,
) {
    let mut attacks = pawn(side, from_sq).bits();
    let ep_square = pos.ep_square();

    while attacks != 0 {
        let to_sq = pop_lsb(&mut attacks);

        if ep_square != Square::NONE && to_sq == ep_square {
            moves.push(Move::en_passant(from_sq, to_sq));
            continue;
        }

        let target = pos.piece_at(to_sq);
        if target != Piece::none() && target.color() != side {
            if (promotion_start..promotion_end).contains(&(to_sq.index() as i32)) {
                add_promotions(from_sq, to_sq, side, moves);
            } else {
                moves.push(Move::normal(from_sq, to_sq));
            }
        }
    }
}

trait PieceMovement {
    fn piece_type() -> PieceType;
    fn get_attacks(from_sq: Square, occupancy: Bitboard) -> Bitboard;
}

struct Knight;
impl PieceMovement for Knight {
    fn piece_type() -> PieceType {
        PieceType::Knight
    }
    fn get_attacks(from_sq: Square, _: Bitboard) -> Bitboard {
        knight(from_sq)
    }
}

struct Bishop;
impl PieceMovement for Bishop {
    fn piece_type() -> PieceType {
        PieceType::Bishop
    }
    fn get_attacks(from_sq: Square, occupancy: Bitboard) -> Bitboard {
        bishop(from_sq, occupancy)
    }
}

struct Rook;
impl PieceMovement for Rook {
    fn piece_type() -> PieceType {
        PieceType::Rook
    }
    fn get_attacks(from_sq: Square, occupancy: Bitboard) -> Bitboard {
        rook(from_sq, occupancy)
    }
}

struct Queen;
impl PieceMovement for Queen {
    fn piece_type() -> PieceType {
        PieceType::Queen
    }
    fn get_attacks(from_sq: Square, occupancy: Bitboard) -> Bitboard {
        queen(from_sq, occupancy)
    }
}

struct King;
impl PieceMovement for King {
    fn piece_type() -> PieceType {
        PieceType::King
    }
    fn get_attacks(from_sq: Square, _: Bitboard) -> Bitboard {
        king(from_sq)
    }
}

fn generate_piece_moves<P: PieceMovement>(
    pos: &Position,
    side: Color,
    occupancy: Bitboard,
    moves: &mut ArrayVec<Move, 256>,
) {
    let mut pieces = pos.pieces_bb_color(side, P::piece_type()).bits();

    while pieces != 0 {
        let from_sq = pop_lsb(&mut pieces);
        let mut targets = P::get_attacks(from_sq, occupancy).bits();

        while targets != 0 {
            let to_sq = pop_lsb(&mut targets);
            let target = pos.piece_at(to_sq);

            if target == Piece::none() || target.color() != side {
                moves.push(Move::normal(from_sq, to_sq));
            }
        }
    }
}
fn generate_castling_moves(pos: &Position, side: Color, moves: &mut ArrayVec<Move, 256>) {
    let king_sq = pos.king_sq(side);

    // Can't castle if in check
    if pieces_attacking_square(king_sq, side, pos).bits() != 0 {
        return;
    }

    match side {
        #[rustfmt::skip]
        Color::White => {
            try_castle(pos, side, moves, CastlingRights::WHITE_KING_SIDE, king_sq, Square::H1);
            try_castle(pos, side, moves, CastlingRights::WHITE_QUEEN_SIDE, king_sq, Square::A1);
        }
        #[rustfmt::skip]
        Color::Black => {
            try_castle(pos, side, moves, CastlingRights::BLACK_KING_SIDE, king_sq, Square::H8);
            try_castle(pos, side, moves, CastlingRights::BLACK_QUEEN_SIDE, king_sq, Square::A8);
        }
    }
}

fn try_castle(
    pos: &Position,
    side: Color,
    moves: &mut ArrayVec<Move, 256>,
    castle_right: CastlingRights,
    king_sq: Square,
    rook_sq: Square,
) {
    let rights = pos.castling_rights();
    if !rights.contains(castle_right) {
        return;
    }

    // Determine squares based on rook position
    #[rustfmt::skip]
    let (check_path_squares, path_squares) = match rook_sq {
        Square::H1 => (&[Square::F1, Square::G1][..], &[Square::F1, Square::G1][..]),
        Square::A1 => (&[Square::C1, Square::D1][..], &[Square::B1, Square::C1, Square::D1][..]),
        Square::H8 => (&[Square::F8, Square::G8][..], &[Square::F8, Square::G8][..]),
        Square::A8 => (&[Square::C8, Square::D8][..], &[Square::B8, Square::C8, Square::D8][..]),
        _ => return,
    };

    for &sq in path_squares {
        if pos.piece_at(sq) != Piece::none() {
            return;
        }
    }

    for &sq in check_path_squares {
        if pieces_attacking_square(sq, side, pos).bits() != 0 {
            return;
        }
    }

    moves.push(Move::castle(king_sq, rook_sq));
}

fn add_promotions(from_sq: Square, to_sq: Square, side: Color, moves: &mut ArrayVec<Move, 256>) {
    for &piece_type in PROMOTION_PIECES.iter() {
        moves.push(Move::promotion(
            from_sq,
            to_sq,
            Piece::new(piece_type, side),
        ));
    }
}

fn pieces_attacking_square(sq: Square, c: Color, pos: &Position) -> Bitboard {
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

#[rustfmt::skip]
static PAWN_ATTACKS: [[u64; 64]; 2] = [
    // White
    [
        0x200, 0x500, 0xa00, 0x1400,
        0x2800, 0x5000, 0xa000, 0x4000,
        0x20000, 0x50000, 0xa0000, 0x140000,
        0x280000, 0x500000, 0xa00000, 0x400000,
        0x2000000, 0x5000000, 0xa000000, 0x14000000,
        0x28000000, 0x50000000, 0xa0000000, 0x40000000,
        0x200000000, 0x500000000, 0xa00000000, 0x1400000000,
        0x2800000000, 0x5000000000, 0xa000000000, 0x4000000000,
        0x20000000000, 0x50000000000, 0xa0000000000, 0x140000000000,
        0x280000000000, 0x500000000000, 0xa00000000000, 0x400000000000,
        0x2000000000000, 0x5000000000000, 0xa000000000000, 0x14000000000000,
        0x28000000000000, 0x50000000000000, 0xa0000000000000, 0x40000000000000,
        0x200000000000000, 0x500000000000000, 0xa00000000000000, 0x1400000000000000,
        0x2800000000000000, 0x5000000000000000, 0xa000000000000000, 0x4000000000000000,
        0x0, 0x0, 0x0, 0x0,
        0x0, 0x0, 0x0, 0x0 
    ],
    // Black
    [ 
        0x0, 0x0, 0x0, 0x0,
        0x0, 0x0, 0x0, 0x0,
        0x2, 0x5, 0xa, 0x14,
        0x28, 0x50, 0xa0, 0x40,
        0x200, 0x500, 0xa00, 0x1400,
        0x2800, 0x5000, 0xa000, 0x4000,
        0x20000, 0x50000, 0xa0000, 0x140000,
        0x280000, 0x500000, 0xa00000, 0x400000,
        0x2000000, 0x5000000, 0xa000000, 0x14000000,
        0x28000000, 0x50000000, 0xa0000000, 0x40000000,
        0x200000000, 0x500000000, 0xa00000000, 0x1400000000,
        0x2800000000, 0x5000000000, 0xa000000000, 0x4000000000,
        0x20000000000, 0x50000000000, 0xa0000000000, 0x140000000000,
        0x280000000000, 0x500000000000, 0xa00000000000, 0x400000000000,
        0x2000000000000, 0x5000000000000, 0xa000000000000, 0x14000000000000,
        0x28000000000000, 0x50000000000000, 0xa0000000000000, 0x40000000000000
    ],
];

#[rustfmt::skip]
static KNIGHT_ATTACKS: [u64; 64] = [
    0x0000000000020400, 0x0000000000050800, 0x00000000000A1100, 0x0000000000142200, 0x0000000000284400,
    0x0000000000508800, 0x0000000000A01000, 0x0000000000402000, 0x0000000002040004, 0x0000000005080008,
    0x000000000A110011, 0x0000000014220022, 0x0000000028440044, 0x0000000050880088, 0x00000000A0100010,
    0x0000000040200020, 0x0000000204000402, 0x0000000508000805, 0x0000000A1100110A, 0x0000001422002214,
    0x0000002844004428, 0x0000005088008850, 0x000000A0100010A0, 0x0000004020002040, 0x0000020400040200,
    0x0000050800080500, 0x00000A1100110A00, 0x0000142200221400, 0x0000284400442800, 0x0000508800885000,
    0x0000A0100010A000, 0x0000402000204000, 0x0002040004020000, 0x0005080008050000, 0x000A1100110A0000,
    0x0014220022140000, 0x0028440044280000, 0x0050880088500000, 0x00A0100010A00000, 0x0040200020400000,
    0x0204000402000000, 0x0508000805000000, 0x0A1100110A000000, 0x1422002214000000, 0x2844004428000000,
    0x5088008850000000, 0xA0100010A0000000, 0x4020002040000000, 0x0400040200000000, 0x0800080500000000,
    0x1100110A00000000, 0x2200221400000000, 0x4400442800000000, 0x8800885000000000, 0x100010A000000000,
    0x2000204000000000, 0x0004020000000000, 0x0008050000000000, 0x00110A0000000000, 0x0022140000000000,
    0x0044280000000000, 0x0088500000000000, 0x0010A00000000000, 0x0020400000000000
];

#[rustfmt::skip]
static KING_ATTACKS: [u64; 64] = [
           0x0000000000000302, 0x0000000000000705, 0x0000000000000E0A, 0x0000000000001C14, 0x0000000000003828,
        0x0000000000007050, 0x000000000000E0A0, 0x000000000000C040, 0x0000000000030203, 0x0000000000070507,
        0x00000000000E0A0E, 0x00000000001C141C, 0x0000000000382838, 0x0000000000705070, 0x0000000000E0A0E0,
        0x0000000000C040C0, 0x0000000003020300, 0x0000000007050700, 0x000000000E0A0E00, 0x000000001C141C00,
        0x0000000038283800, 0x0000000070507000, 0x00000000E0A0E000, 0x00000000C040C000, 0x0000000302030000,
        0x0000000705070000, 0x0000000E0A0E0000, 0x0000001C141C0000, 0x0000003828380000, 0x0000007050700000,
        0x000000E0A0E00000, 0x000000C040C00000, 0x0000030203000000, 0x0000070507000000, 0x00000E0A0E000000,
        0x00001C141C000000, 0x0000382838000000, 0x0000705070000000, 0x0000E0A0E0000000, 0x0000C040C0000000,
        0x0003020300000000, 0x0007050700000000, 0x000E0A0E00000000, 0x001C141C00000000, 0x0038283800000000,
        0x0070507000000000, 0x00E0A0E000000000, 0x00C040C000000000, 0x0302030000000000, 0x0705070000000000,
        0x0E0A0E0000000000, 0x1C141C0000000000, 0x3828380000000000, 0x7050700000000000, 0xE0A0E00000000000,
        0xC040C00000000000, 0x0203000000000000, 0x0507000000000000, 0x0A0E000000000000, 0x141C000000000000,
        0x2838000000000000, 0x5070000000000000, 0xA0E0000000000000, 0x40C0000000000000
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

    fn split_perft(fen: &str, depth: u32) -> u64 {
        let pos = Position::from_fen(fen).unwrap();
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
        assert_eq!(split_perft(STARTPOS, 1), 20);
    }

    #[test]
    fn test_perft_startpos_depth_2() {
        assert_eq!(split_perft(STARTPOS, 2), 400);
    }

    #[test]
    fn test_perft_startpos_depth_3() {
        assert_eq!(split_perft(STARTPOS, 3), 8902);
    }

    #[test]
    fn test_perft_startpos_depth_4() {
        assert_eq!(split_perft(STARTPOS, 4), 197281);
    }

    #[test]
    fn test_perft_startpos_depth_5() {
        assert_eq!(split_perft(STARTPOS, 5), 4865609);
    }

    #[test]
    fn test_perft_startpos_depth_7() {
        assert_eq!(
            split_perft(
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
                7
            ),
            3195901860
        );
    }

    #[test]
    fn test_perft_custom_position_1() {
        assert_eq!(
            split_perft(
                "rnbqkbnr/ppp1pppp/3p4/8/8/2P5/PP1PPPPP/RNBQKBNR w KQkq - 0 2",
                1
            ),
            21
        );
    }

    #[test]
    fn test_perft_custom_position_2() {
        assert_eq!(
            split_perft(
                "rnbqkbnr/pppppppp/8/8/8/2P5/PP1PPPPP/RNBQKBNR b KQkq - 0 1",
                2
            ),
            420
        );
    }

    #[test]
    fn test_perft_castle_position() {
        assert_eq!(
            split_perft(
                "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
                1
            ),
            48
        );
    }

    #[test]
    fn test_perft_complex_position_1() {
        assert_eq!(
            split_perft(
                "r3k2r/p1ppqpb1/bnN1pnp1/3P4/1p2P3/2N2Q1p/PPPBBPPP/R3K2R b KQkq - 1 1",
                1
            ),
            41
        );
    }

    #[test]
    fn test_perft_complex_position_2() {
        assert_eq!(
            split_perft(
                "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/P1N2Q2/1PPBBPpP/R3K2R w KQkq - 0 2",
                1
            ),
            48
        );
    }

    #[test]
    fn test_perft_complex_position_32() {
        assert_eq!(
            split_perft(
                "r3k2r/p1pNqpb1/bn2pnp1/3P4/1p2P3/2N2Q1p/PPPBBPPP/R3K2R b KQkq - 0 1",
                1
            ),
            45
        );
    }

    #[test]
    fn test_perft_complex_position_3() {
        assert_eq!(
            split_perft(
                "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
                2
            ),
            2039
        );
    }

    #[test]
    fn test_perft_complex_position_4() {
        assert_eq!(
            split_perft(
                "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/P1N2Q1p/1PPBBPPP/R3K2R b KQkq - 0 1",
                2
            ),
            2186
        );
    }

    #[test]
    fn test_perft_complex_position_25() {
        assert_eq!(
            split_perft(
                "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/5Q2/PPPBBPpP/RN2K2R w KQkq - 0 2",
                1
            ),
            47
        );
    }

    #[test]
    fn test_perft_complex_position_5() {
        assert_eq!(
            split_perft(
                "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
                3
            ),
            97862
        );
    }

    #[test]
    fn test_perft_complex_position_6() {
        assert_eq!(
            split_perft(
                "r3k2r/p1p1qpb1/bn1ppnp1/1B1PN3/1p2P3/P1N2Q1p/1PPB1PPP/R3K2R b KQkq - 1 2",
                1
            ),
            7
        );
    }

    #[test]
    fn test_perft_complex_position_7() {
        assert_eq!(
            split_perft(
                "r3k2r/p1p1qpb1/bn1ppnp1/3PN3/1p2P3/P1N2Q1p/1PPBBPPP/R3K2R w KQkq - 0 2",
                2
            ),
            2135
        );
    }

    #[test]
    fn test_perft_complex_position_8() {
        assert_eq!(
            split_perft(
                "r3k2r/p1ppqpb1/bn2p1p1/3PN3/1p2n3/P1N2Q1p/1PPBBPPP/R3K2R w KQkq - 0 2",
                2
            ),
            2717
        );
    }

    #[test]
    fn test_perft_complex_position_9() {
        assert_eq!(
            split_perft(
                "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/P1N2Q1p/1PPBBPPP/R3K2R b KQkq - 0 1",
                3
            ),
            94405
        );
    }

    #[test]
    fn test_perft_complex_position_10() {
        assert_eq!(
            split_perft(
                "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
                4
            ),
            4085603
        );
    }

    #[test]
    fn test_perft_complex_position_11() {
        assert_eq!(
            split_perft(
                "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
                5
            ),
            193690690
        );
    }

    #[test]
    fn test_perft_endgame_position() {
        assert_eq!(
            split_perft("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", 7),
            178633661
        );
    }

    #[test]
    fn test_perft_tactical_position_1() {
        assert_eq!(
            split_perft(
                "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
                6
            ),
            706045033
        );
    }

    #[test]
    fn test_perft_tactical_position_2() {
        assert_eq!(
            split_perft(
                "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
                5
            ),
            89941194
        );
    }

    #[test]
    fn test_perft_tactical_position_3() {
        assert_eq!(
            split_perft(
                "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 1",
                5
            ),
            164075551
        );
    }
}
