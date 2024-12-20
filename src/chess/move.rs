use crate::chess::{
    castling_rights::CastleType,
    color::Color,
    coords::{File, Square},
    piece::Piece,
    piecetype::PieceType,
};
use crate::compressed_move::CompressedMove;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MoveType {
    Normal,
    Promotion,
    Castle,
    EnPassant,
}

impl MoveType {
    pub const fn from_ordinal(ordinal: u8) -> Self {
        match ordinal {
            0 => Self::Normal,
            1 => Self::Promotion,
            2 => Self::Castle,
            3 => Self::EnPassant,
            _ => panic!("Invalid ordinal for MoveType"),
        }
    }

    pub const fn ordinal(&self) -> u8 {
        match self {
            Self::Normal => 0,
            Self::Promotion => 1,
            Self::Castle => 2,
            Self::EnPassant => 3,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    move_type: MoveType,
    promoted_piece: Piece,
}

impl Move {
    pub fn compress(&self) -> CompressedMove {
        CompressedMove::from_move(*self)
    }

    pub fn new(from: Square, to: Square, move_type: MoveType, promoted_piece: Piece) -> Self {
        debug_assert!(from.index() < 64);
        debug_assert!(to.index() < 64);

        Self {
            from,
            to,
            move_type,
            promoted_piece,
        }
    }

    pub const fn null() -> Self {
        Self {
            from: Square::NONE,
            to: Square::NONE,
            move_type: MoveType::Normal,
            promoted_piece: Piece::none(),
        }
    }

    pub const fn mtype(&self) -> MoveType {
        self.move_type
    }

    pub const fn promoted_piece(&self) -> Piece {
        self.promoted_piece
    }

    pub const fn from(&self) -> Square {
        self.from
    }

    pub const fn to(&self) -> Square {
        self.to
    }

    pub const fn normal(from: Square, to: Square) -> Self {
        Self {
            from,
            to,
            move_type: MoveType::Normal,
            promoted_piece: Piece::none(),
        }
    }

    pub const fn en_passant(from: Square, to: Square) -> Self {
        Self {
            from,
            to,
            move_type: MoveType::EnPassant,
            promoted_piece: Piece::none(),
        }
    }

    pub const fn promotion(from: Square, to: Square, piece: Piece) -> Self {
        Self {
            from,
            to,
            move_type: MoveType::Promotion,
            promoted_piece: piece,
        }
    }

    pub const fn castle(from: Square, to: Square) -> Self {
        Self {
            from,
            to,
            move_type: MoveType::Castle,
            promoted_piece: Piece::none(),
        }
    }

    pub fn from_castle(ct: CastleType, stm: Color) -> Self {
        match ct {
            CastleType::Short => {
                if stm == Color::White {
                    // Self::castle(Square::E1, Square::G1)
                    Self::castle(Square::E1, Square::H1)
                } else {
                    // Self::castle(Square::E8, Square::G8)
                    Self::castle(Square::E8, Square::H8)
                }
            }
            CastleType::Long => {
                if stm == Color::White {
                    // Self::castle(Square::E1, Square::C1)
                    Self::castle(Square::E1, Square::A1)
                } else {
                    // Self::castle(Square::E8, Square::C8)
                    Self::castle(Square::E8, Square::A8)
                }
            }
        }
    }

    pub fn castle_type(&self) -> CastleType {
        if self.to.file() == File::H {
            CastleType::Short
        } else {
            CastleType::Long
        }
    }

    pub fn as_uci(&self) -> String {
        let mut uci = format!("{}{}", self.from, self.to);

        if self.move_type == MoveType::Promotion {
            uci.push(match self.promoted_piece.piece_type() {
                PieceType::Queen => 'q',
                PieceType::Rook => 'r',
                PieceType::Bishop => 'b',
                PieceType::Knight => 'n',
                _ => panic!("Invalid promotion piece"),
            });
        }

        uci
    }
}

impl Default for Move {
    fn default() -> Self {
        Self::null()
    }
}
