#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
    None,
}

impl PieceType {
    /// Create a piece type from an ordinal, must be in the range [0, 6]
    #[inline(always)]
    pub const fn from_ordinal(value: u8) -> Self {
        match value {
            0 => Self::Pawn,
            1 => Self::Knight,
            2 => Self::Bishop,
            3 => Self::Rook,
            4 => Self::Queen,
            5 => Self::King,
            6 => Self::None,
            _ => panic!("Invalid ordinal value for PieceType"),
        }
    }

    /// 0 for Pawn, 1 for Knight, 2 for Bishop,
    /// 3 for Rook, 4 for Queen, 5 for King,
    /// 6 for None
    #[inline(always)]
    pub const fn ordinal(&self) -> u8 {
        *self as u8
    }
}
