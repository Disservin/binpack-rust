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
        debug_assert!(value < 7);
        unsafe { std::mem::transmute(value) }
    }

    /// 0 for Pawn, 1 for Knight, 2 for Bishop,
    /// 3 for Rook, 4 for Queen, 5 for King,
    /// 6 for None
    #[inline(always)]
    pub const fn ordinal(&self) -> u8 {
        *self as u8
    }
}
