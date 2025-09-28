use std::ops::{BitAndAssign, BitOrAssign, Not};

use super::color::Color;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CastleType {
    Short,
    Long,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CastlingRights(u8);

impl CastlingRights {
    pub const NONE: Self = Self(0x0);
    pub const WHITE_KING_SIDE: Self = Self(0x1);
    pub const WHITE_QUEEN_SIDE: Self = Self(0x2);
    pub const BLACK_KING_SIDE: Self = Self(0x4);
    pub const BLACK_QUEEN_SIDE: Self = Self(0x8);
    pub const WHITE: Self = Self(Self::WHITE_KING_SIDE.0 | Self::WHITE_QUEEN_SIDE.0);
    pub const BLACK: Self = Self(Self::BLACK_KING_SIDE.0 | Self::BLACK_QUEEN_SIDE.0);
    pub const ALL: Self = Self(
        Self::WHITE_KING_SIDE.0
            | Self::WHITE_QUEEN_SIDE.0
            | Self::BLACK_KING_SIDE.0
            | Self::BLACK_QUEEN_SIDE.0,
    );

    /// Create an empty set of castling rights.
    pub fn empty() -> Self {
        Self(0)
    }

    /// Check if it contains a specific castling right.
    pub fn contains(&self, other: CastlingRights) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Check how many castling rights are set, max is 4, min is 0.
    pub fn count_ones(&self) -> u32 {
        self.0.count_ones()
    }

    /// Get all castling rights for a specific color.
    #[allow(clippy::self_named_constructors)]
    pub fn castling_rights(color: Color) -> Self {
        match color {
            Color::White => Self::WHITE,
            Color::Black => Self::BLACK,
        }
    }
}

impl std::ops::BitAnd for CastlingRights {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

pub struct CastlingTraits;

impl CastlingTraits {
    pub fn castling_rights(color: Color, castle_type: CastleType) -> CastlingRights {
        match (color, castle_type) {
            (Color::White, CastleType::Short) => CastlingRights::WHITE_KING_SIDE,
            (Color::White, CastleType::Long) => CastlingRights::WHITE_QUEEN_SIDE,
            (Color::Black, CastleType::Short) => CastlingRights::BLACK_KING_SIDE,
            (Color::Black, CastleType::Long) => CastlingRights::BLACK_QUEEN_SIDE,
        }
    }
}

impl Not for CastlingRights {
    type Output = Self;

    fn not(self) -> Self {
        Self(!self.0)
    }
}

impl BitAndAssign for CastlingRights {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitOrAssign for CastlingRights {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}
