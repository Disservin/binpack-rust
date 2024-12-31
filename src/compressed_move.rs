use crate::chess::{
    color::Color,
    coords::{Rank, Square},
    piece::Piece,
    piecetype::PieceType,
    r#move::{Move, MoveType},
};

/// A compressed move representation, using 16 bits.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct CompressedMove {
    // from most significant bits
    // 2 bits for move type
    // 6 bits for from square
    // 6 bits for to square
    // 2 bits for promoted piece type
    //    0 if not a promotion
    packed: u16,
}

impl CompressedMove {
    const SQUARE_MASK: u16 = 0b111111;
    const PROMOTED_PIECE_TYPE_MASK: u16 = 0b11;

    pub fn byte_size() -> usize {
        std::mem::size_of::<CompressedMove>()
    }

    pub fn read_from_big_endian(data: &[u8]) -> Self {
        Self {
            packed: ((data[0] as u16) << 8) | (data[1] as u16),
        }
    }

    pub const fn new() -> Self {
        Self { packed: 0 }
    }

    // pub const fn from_ordinal(data: u16) -> Self {
    //     Self { packed: data }
    // }

    // move must be either valid or a null move
    // pub fn from_move(move_: Move) -> Self {
    //     let mut packed = 0;

    //     // else null move
    //     if move_.from() != move_.to() {
    //         debug_assert!(move_.from() != Square::NONE);
    //         debug_assert!(move_.to() != Square::NONE);

    //         packed = ((move_.mtype() as u16) << (16 - 2))
    //             | ((move_.from().index() as u16) << (16 - 2 - 6))
    //             | ((move_.to().index() as u16) << (16 - 2 - 6 - 6));

    //         if move_.mtype() == MoveType::Promotion {
    //             debug_assert!(move_.promoted_piece() != Piece::none());

    //             packed |= (move_.promoted_piece().piece_type() as u16) - (PieceType::Knight as u16);
    //         } else {
    //             debug_assert!(move_.promoted_piece() == Piece::none());
    //         }
    //     }

    //     Self { packed }
    // }

    // pub fn write_to_big_endian(&self, data: &mut [u8]) {
    //     data[0] = (self.packed >> 8) as u8;
    //     data[1] = (self.packed & 0xFF) as u8;
    // }

    // pub const fn packed(&self) -> u16 {
    //     self.packed
    // }

    pub const fn move_type(&self) -> MoveType {
        // Assuming MoveType implements From<u16>
        MoveType::from_ordinal((self.packed >> (16 - 2)) as u8)
    }

    pub const fn from(&self) -> Square {
        Square::new(((self.packed >> (16 - 2 - 6)) & Self::SQUARE_MASK) as u32)
    }

    pub const fn to(&self) -> Square {
        Square::new(((self.packed >> (16 - 2 - 6 - 6)) & Self::SQUARE_MASK) as u32)
    }

    pub fn promoted_piece(&self) -> Piece {
        if self.move_type() == MoveType::Promotion {
            let color = if self.to().rank() == Rank::FIRST {
                Color::Black
            } else {
                Color::White
            };

            let piece_type = PieceType::from_ordinal(
                ((self.packed & Self::PROMOTED_PIECE_TYPE_MASK) as u8) + (PieceType::Knight as u8),
            );

            Piece::new(piece_type, color)
        } else {
            Piece::none()
        }
    }

    pub fn decompress(&self) -> Move {
        if self.packed == 0 {
            Move::null()
        } else {
            let move_type = self.move_type();
            let from = self.from();
            let to = self.to();
            let promoted_piece = self.promoted_piece();

            debug_assert!(from != Square::NONE);
            debug_assert!(to != Square::NONE);

            Move::new(from, to, move_type, promoted_piece)
        }
    }
}

impl Default for CompressedMove {
    fn default() -> Self {
        Self::new()
    }
}
