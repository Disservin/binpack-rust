use std::{
    fmt::{self},
    ops::{Add, Sub},
};

use super::color::Color;

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct FlatSquareOffset {
    value: i8,
}

impl FlatSquareOffset {
    pub const fn new(files: i32, ranks: i32) -> Self {
        const FILE_CARDINALITY: i32 = 8;

        let offset = files + ranks * FILE_CARDINALITY;

        debug_assert!(offset >= i8::MIN as i32);
        debug_assert!(offset <= i8::MAX as i32);

        Self {
            value: offset as i8,
        }
    }

    pub const fn default() -> Self {
        Self { value: 0 }
    }

    pub const fn neg(&self) -> Self {
        Self { value: -self.value }
    }

    pub fn forward(color: Color) -> Self {
        if color == Color::White {
            Self::new(0, 1)
        } else {
            Self::new(0, -1)
        }
    }
}

impl std::ops::Neg for FlatSquareOffset {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self { value: -self.value }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Square {
    index: u32,
}

impl Square {
    pub const NONE: Self = Self { index: 64 };
    pub const A1: Self = Self { index: 0 };
    pub const B1: Self = Self { index: 1 };
    pub const C1: Self = Self { index: 2 };
    pub const D1: Self = Self { index: 3 };
    pub const E1: Self = Self { index: 4 };
    pub const F1: Self = Self { index: 5 };
    pub const G1: Self = Self { index: 6 };
    pub const H1: Self = Self { index: 7 };
    pub const A8: Self = Self { index: 56 };
    pub const B8: Self = Self { index: 57 };
    pub const C8: Self = Self { index: 58 };
    pub const D8: Self = Self { index: 59 };
    pub const E8: Self = Self { index: 60 };
    pub const F8: Self = Self { index: 61 };
    pub const G8: Self = Self { index: 62 };
    pub const H8: Self = Self { index: 63 };

    #[must_use]
    pub const fn new(index: u32) -> Self {
        debug_assert!(index < 64);
        Self { index }
    }

    #[must_use]
    pub const fn from_i32(index: i32) -> Self {
        debug_assert!(index >= 0 && index < 64);
        Self {
            index: index as u32,
        }
    }

    pub fn from_string(s: &str) -> Option<Self> {
        if s.len() != 2 {
            return None;
        }

        let file = s.chars().nth(0).unwrap();
        let rank = s.chars().nth(1).unwrap();

        let file = file as i32 - 'a' as i32;
        let rank = rank as i32 - '1' as i32;

        if Self::is_valid(rank as i64, file as i64) {
            Some(Self::from_rank_file(rank as i64, file as i64))
        } else {
            None
        }
    }

    #[must_use]
    pub const fn index(self) -> u32 {
        self.index
    }

    #[must_use]
    pub const fn file(self) -> File {
        File::new(self.index & 7)
    }

    #[must_use]
    pub const fn rank(self) -> Rank {
        Rank::new(self.index >> 3)
    }

    #[must_use]
    pub fn offset(self, files: i32, ranks: i32) -> Option<Self> {
        const FILE_CARDINALITY: i32 = 8;
        let offset = files + ranks * FILE_CARDINALITY;
        let new_index = self.index as i32 + offset;

        (0..64).contains(&new_index).then_some(Self {
            index: new_index as u32,
        })
    }

    /// Check if the square is within bounds
    #[must_use]
    pub const fn is_valid(r: i64, f: i64) -> bool {
        r >= 0 && r < 8 && f >= 0 && f < 8
    }

    #[must_use]
    pub const fn from_rank_file(r: i64, f: i64) -> Self {
        if Self::is_valid(r, f) {
            Self {
                index: (r * 8 + f) as u32,
            }
        } else {
            Self::NONE
        }
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.file(), self.rank())
    }
}

impl Add<Square> for Square {
    type Output = Square;

    fn add(self, rhs: Square) -> Square {
        Self {
            index: (self.index as i32 + rhs.index as i32) as u32,
        }
    }
}

impl Add<FlatSquareOffset> for Square {
    type Output = Square;

    fn add(self, rhs: FlatSquareOffset) -> Square {
        Self {
            index: (self.index as i32 + rhs.value as i32) as u32,
        }
    }
}

impl Sub<Square> for Square {
    type Output = Square;

    fn sub(self, rhs: Square) -> Square {
        let res = self.index as i32 - rhs.index as i32;
        debug_assert!((0..64).contains(&res));

        Self {
            index: (res) as u32,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct File {
    index: u32,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Rank {
    index: u32,
}

impl File {
    pub const A: Self = Self { index: 0 };
    pub const B: Self = Self { index: 1 };
    pub const C: Self = Self { index: 2 };
    pub const D: Self = Self { index: 3 };
    pub const E: Self = Self { index: 4 };
    pub const F: Self = Self { index: 5 };
    pub const G: Self = Self { index: 6 };
    pub const H: Self = Self { index: 7 };

    pub const fn new(index: u32) -> Self {
        Self { index }
    }

    pub const fn from_u32(index: u32) -> Self {
        Self { index }
    }
}

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", (b'a' + self.index as u8) as char)
    }
}

impl Rank {
    pub const FIRST: Self = Self { index: 0 };
    pub const SECOND: Self = Self { index: 1 };
    pub const THIRD: Self = Self { index: 2 };
    pub const FOURTH: Self = Self { index: 3 };
    pub const FIFTH: Self = Self { index: 4 };
    pub const SIXTH: Self = Self { index: 5 };
    pub const SEVENTH: Self = Self { index: 6 };
    pub const EIGHTH: Self = Self { index: 7 };

    pub const fn new(index: u32) -> Self {
        Self { index }
    }

    pub const fn from_u32(index: u32) -> Self {
        Self { index }
    }

    pub fn last_pawn_rank(color: Color) -> Self {
        if color == Color::White {
            Self::SEVENTH
        } else {
            Self::SECOND
        }
    }
}

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.index + 1)
    }
}
