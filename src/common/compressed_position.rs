use crate::chess::{
    bitboard::Bitboard,
    castling_rights::CastlingRights,
    color::Color,
    coords::{FlatSquareOffset, Rank, Square},
    piece::Piece,
    piecetype::PieceType,
    position::Position,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CompressedPosition {
    occupied: Bitboard,
    packed_state: [u8; 16],
}

impl CompressedPosition {
    pub fn byte_size() -> usize {
        std::mem::size_of::<CompressedPosition>()
    }

    pub fn read_from_big_endian(data: &[u8]) -> Self {
        debug_assert!(data.len() >= 24);

        let occupied = Bitboard::new(
            ((data[0] as u64) << 56)
                | ((data[1] as u64) << 48)
                | ((data[2] as u64) << 40)
                | ((data[3] as u64) << 32)
                | ((data[4] as u64) << 24)
                | ((data[5] as u64) << 16)
                | ((data[6] as u64) << 8)
                | (data[7] as u64),
        );

        let mut packed_state = [0u8; 16];
        packed_state.copy_from_slice(&data[8..24]);

        Self {
            occupied,
            packed_state,
        }
    }

    pub fn write_to_big_endian(&self, data: &mut [u8]) {
        let occupied = self.occupied.bits();
        data[0] = (occupied >> 56) as u8;
        data[1] = ((occupied >> 48) & 0xFF) as u8;
        data[2] = ((occupied >> 40) & 0xFF) as u8;
        data[3] = ((occupied >> 32) & 0xFF) as u8;
        data[4] = ((occupied >> 24) & 0xFF) as u8;
        data[5] = ((occupied >> 16) & 0xFF) as u8;
        data[6] = ((occupied >> 8) & 0xFF) as u8;
        data[7] = (occupied & 0xFF) as u8;
        data[8..24].copy_from_slice(&self.packed_state[..16]);
    }

    pub fn decompress(&self) -> Position {
        let mut pos = Position::empty();
        pos.set_castling_rights(CastlingRights::NONE);

        let mut decompress_piece = |sq: Square, nibble: u8| {
            match nibble {
                0..=11 => {
                    pos.place(Piece::from_id(nibble as i32), sq);
                }
                12 => {
                    let rank = sq.rank();
                    if rank == Rank::FOURTH {
                        pos.place(Piece::WHITE_PAWN, sq);
                        pos.set_ep_square_unchecked(sq + FlatSquareOffset::new(0, -1));
                    } else {
                        // rank == Rank::FIFTH
                        pos.place(Piece::BLACK_PAWN, sq);
                        pos.set_ep_square_unchecked(sq + FlatSquareOffset::new(0, 1));
                    }
                }
                13 => {
                    pos.place(Piece::WHITE_ROOK, sq);
                    if sq == Square::A1 {
                        pos.add_castling_rights(CastlingRights::WHITE_QUEEN_SIDE);
                    } else {
                        // sq == Square::H1
                        pos.add_castling_rights(CastlingRights::WHITE_KING_SIDE);
                    }
                }
                14 => {
                    pos.place(Piece::BLACK_ROOK, sq);
                    if sq == Square::A8 {
                        pos.add_castling_rights(CastlingRights::BLACK_QUEEN_SIDE);
                    } else {
                        // sq == Square::H8
                        pos.add_castling_rights(CastlingRights::BLACK_KING_SIDE);
                    }
                }
                15 => {
                    pos.place(Piece::BLACK_KING, sq);
                    pos.set_side_to_move(Color::Black);
                }
                _ => unreachable!(),
            }
        };

        let mut squares_iter = self.occupied.iter();
        for chunk in self.packed_state.iter() {
            if let Some(sq) = squares_iter.next() {
                decompress_piece(sq, chunk & 0xF);
            } else {
                break;
            }

            if let Some(sq) = squares_iter.next() {
                decompress_piece(sq, chunk >> 4);
            } else {
                break;
            }
        }

        pos
    }

    pub fn compress(pos: &Position) -> Self {
        let mut compressed = CompressedPosition {
            occupied: pos.occupied(),
            packed_state: [0u8; 16],
        };

        let pack_piece = |sq: Square| -> u8 {
            let piece = pos.piece_at(sq);
            let piece_id = piece.id();

            // Special case: pawn with en passant
            if piece.piece_type() == PieceType::Pawn {
                let ep_sq = pos.ep_square();
                if ep_sq != Square::NONE
                    && ((piece.color() == Color::White
                        && sq.rank() == Rank::FOURTH
                        && ep_sq == sq + FlatSquareOffset::new(0, -1))
                        || (piece.color() == Color::Black
                            && sq.rank() == Rank::FIFTH
                            && ep_sq == sq + FlatSquareOffset::new(0, 1)))
                {
                    return 12;
                }
            }

            // Special case: rooks with castling rights
            if piece == Piece::WHITE_ROOK
                && ((sq == Square::A1
                    && pos
                        .castling_rights()
                        .contains(CastlingRights::WHITE_QUEEN_SIDE))
                    || (sq == Square::H1
                        && pos
                            .castling_rights()
                            .contains(CastlingRights::WHITE_KING_SIDE)))
            {
                return 13;
            }
            if piece == Piece::BLACK_ROOK
                && ((sq == Square::A8
                    && pos
                        .castling_rights()
                        .contains(CastlingRights::BLACK_QUEEN_SIDE))
                    || (sq == Square::H8
                        && pos
                            .castling_rights()
                            .contains(CastlingRights::BLACK_KING_SIDE)))
            {
                return 14;
            }

            // Special case: black king when black to move
            if piece == Piece::BLACK_KING && pos.side_to_move() == Color::Black {
                return 15;
            }

            piece_id
        };

        let mut idx = 0;
        for (nibble_idx, sq) in compressed.occupied.iter().enumerate() {
            let nibble = pack_piece(sq);
            if nibble_idx % 2 == 0 {
                compressed.packed_state[idx] = nibble;
            } else {
                compressed.packed_state[idx] |= nibble << 4;
                idx += 1;
            }
        }

        compressed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_big_endian() {
        let data = [
            98, 121, 192, 21, 24, 76, 241, 100, 100, 106, 0, 4, 8, 48, 2, 17, 17, 145, 19, 117,
            247, 0, 0, 0,
        ];

        let compressed_pos = CompressedPosition::read_from_big_endian(&data);

        assert_eq!(
            CompressedPosition {
                occupied: Bitboard::new(7095913884733469028),
                packed_state: [100, 106, 0, 4, 8, 48, 2, 17, 17, 145, 19, 117, 247, 0, 0, 0]
            },
            compressed_pos
        );
    }

    #[test]
    fn test_compressed_position() {
        let data = [
            98, 121, 192, 21, 24, 76, 241, 100, 100, 106, 0, 4, 8, 48, 2, 17, 17, 145, 19, 117,
            247, 0, 0, 0,
        ];

        let compressed_pos = CompressedPosition::read_from_big_endian(&data);
        let pos = compressed_pos.decompress();

        assert_eq!(
            pos.fen().unwrap(),
            "1r3rk1/p2qnpb1/6pp/P1p1p3/3nN3/2QP2P1/R3PPBP/2B2RK1 b - - 0 1"
        );
    }

    // #[test]
    // #[should_panic(expected = "range end index 24 out of range for slice of length 23")]
    // fn test_too_small_data() {
    //     let data = [0; 23];

    //     let _ = CompressedPosition::read_from_big_endian(&data).decompress();
    // }

    #[test]
    fn test_write_big_endian() {
        let data = [
            98, 121, 192, 21, 24, 76, 241, 100, 100, 106, 0, 4, 8, 48, 2, 17, 17, 145, 19, 117,
            247, 0, 0, 0,
        ];

        let compressed_pos = CompressedPosition::read_from_big_endian(&data);
        let mut new_data = [0; 24];
        compressed_pos.write_to_big_endian(&mut new_data);

        assert_eq!(data, new_data);
    }

    #[test]
    fn test_compress_decompress() {
        let pos =
            Position::from_fen("1r3rk1/p2qnpb1/6pp/P1p1p3/3nN3/2QP2P1/R3PPBP/2B2RK1 b - - 0 1")
                .unwrap();

        let compressed_pos = CompressedPosition::compress(&pos);
        let decompressed_pos = compressed_pos.decompress();

        assert_eq!(pos, decompressed_pos);
    }

    #[test]
    fn test_compress_decompress_2() {
        let pos =
            Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();

        let compressed_pos = CompressedPosition::compress(&pos);
        let decompressed_pos = compressed_pos.decompress();

        assert_eq!(pos, decompressed_pos);
    }

    #[test]
    fn test_compress_decompress_3() {
        let pos = Position::from_fen("2r3k1/4bpp1/2Q1p2P/p3P3/1p6/4B1P1/P1r2PK1/3R1R2 b - - 0 30")
            .unwrap();

        let compressed_pos = CompressedPosition::compress(&pos);
        let decompressed_pos = compressed_pos.decompress();

        let position_without_fmt =
            Position::from_fen("2r3k1/4bpp1/2Q1p2P/p3P3/1p6/4B1P1/P1r2PK1/3R1R2 b - - 0 1")
                .unwrap();

        assert_eq!(position_without_fmt, decompressed_pos);
    }
}
