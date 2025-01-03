use crate::{
    chess::bitboard::Bitboard,
    chess::castling_rights::CastlingRights,
    chess::color::Color,
    chess::coords::{FlatSquareOffset, Rank, Square},
    chess::piece::Piece,
    chess::position::Position,
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

    pub fn decompress(&self) -> Position {
        let mut pos = Position::new();
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
            pos.fen(),
            "1r3rk1/p2qnpb1/6pp/P1p1p3/3nN3/2QP2P1/R3PPBP/2B2RK1 b - - 0 1"
        );
    }

    #[test]
    #[should_panic(expected = "range end index 24 out of range for slice of length 23")]
    fn test_too_small_data() {
        let data = [0; 23];

        let _ = CompressedPosition::read_from_big_endian(&data).decompress();
    }
}
