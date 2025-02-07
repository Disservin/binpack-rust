use crate::chess::{color::Color, piecetype::PieceType};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Piece {
    /// lowest bit is a color, 7 highest bits are a piece type
    id: u8,
}

/// Piece representation
impl Piece {
    pub const WHITE_PAWN: Piece = Piece::new(PieceType::Pawn, Color::White);
    pub const WHITE_KNIGHT: Piece = Piece::new(PieceType::Knight, Color::White);
    pub const WHITE_BISHOP: Piece = Piece::new(PieceType::Bishop, Color::White);
    pub const WHITE_ROOK: Piece = Piece::new(PieceType::Rook, Color::White);
    pub const WHITE_QUEEN: Piece = Piece::new(PieceType::Queen, Color::White);
    pub const WHITE_KING: Piece = Piece::new(PieceType::King, Color::White);

    pub const BLACK_PAWN: Piece = Piece::new(PieceType::Pawn, Color::Black);
    pub const BLACK_KNIGHT: Piece = Piece::new(PieceType::Knight, Color::Black);
    pub const BLACK_BISHOP: Piece = Piece::new(PieceType::Bishop, Color::Black);
    pub const BLACK_ROOK: Piece = Piece::new(PieceType::Rook, Color::Black);
    pub const BLACK_QUEEN: Piece = Piece::new(PieceType::Queen, Color::Black);
    pub const BLACK_KING: Piece = Piece::new(PieceType::King, Color::Black);
    pub const NONE: Piece = Piece::none();

    /// Create a piece from an id, must be in the range [0, 12]
    pub const fn from_id(id: i32) -> Self {
        debug_assert!(id >= 0 && id < 13);
        Self { id: id as u8 }
    }

    #[inline(always)]
    pub const fn none() -> Self {
        Self::new(PieceType::None, Color::White)
    }

    pub const fn new(piece_type: PieceType, color: Color) -> Self {
        Self {
            id: (piece_type.ordinal() << 1) | color.ordinal(),
        }
    }

    /// Get the piece type of the piece
    #[inline(always)]
    pub const fn piece_type(&self) -> PieceType {
        PieceType::from_ordinal(self.id >> 1)
    }

    /// Get the color of the piece
    #[inline(always)]
    pub const fn color(&self) -> Color {
        Color::from_ordinal(self.id & 1)
    }

    /// Get the piece type and color of the piece
    pub fn parts(&self) -> (PieceType, Color) {
        (self.piece_type(), self.color())
    }

    pub fn id(&self) -> u8 {
        self.id
    }
}
