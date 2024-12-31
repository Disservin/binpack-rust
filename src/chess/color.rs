use std::ops::Not;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
}

impl Color {
    /// White is 0, Black is 1
    pub const fn from_ordinal(value: u8) -> Self {
        match value {
            0 => Color::White,
            1 => Color::Black,
            _ => panic!("Invalid color ordinal"),
        }
    }

    /// White is 0, Black is 1
    pub const fn ordinal(&self) -> u8 {
        match self {
            Color::White => 0,
            Color::Black => 1,
        }
    }
}

impl Not for Color {
    type Output = Color;

    fn not(self) -> Self::Output {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}
