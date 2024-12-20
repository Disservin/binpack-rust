use std::ops::{BitAnd, BitOr, BitOrAssign, Not};

use crate::chess::coords::{File, Rank, Square};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bitboard {
    data: u64,
}

impl Bitboard {
    #[must_use]
    #[inline(always)]
    pub fn count(&self) -> u32 {
        self.data.count_ones()
    }

    pub fn get(&self, index: u32) -> bool {
        self.data & (1 << index) != 0
    }

    pub fn set(&mut self, index: u32, value: bool) {
        if value {
            self.data |= 1 << index;
        } else {
            self.data &= !(1 << index);
        }
    }

    pub fn clear(&mut self) {
        self.data = 0;
    }

    pub fn is_set(&self, index: u32) -> bool {
        self.data & (1 << index) != 0
    }

    pub fn sq_set(&self, index: Square) -> bool {
        self.data & (1 << index.to_u32()) != 0
    }

    pub const fn new(bits: u64) -> Self {
        Bitboard { data: bits }
    }

    pub fn from_u64(data: u64) -> Self {
        Self { data }
    }

    pub fn to_u64(&self) -> u64 {
        self.data
    }

    pub fn from_before(index: u32) -> Self {
        Self {
            data: (1 << index) - 1,
        }
    }

    pub fn from_square(index: Square) -> Self {
        Self {
            data: 1 << index.to_u32(),
        }
    }

    pub fn from_file(index: u32) -> Self {
        Self {
            data: 0x0101010101010101 << index,
        }
    }

    pub fn from_rank(index: u32) -> Self {
        Self {
            data: 0xFF << (index * 8),
        }
    }

    pub fn rank(&self) -> Rank {
        Rank::new((self.data >> 3) as u32)
    }

    pub fn file(&self) -> File {
        File::new((self.data & 7) as u32)
    }

    pub fn iter(&self) -> BitboardIterator {
        BitboardIterator {
            remaining: self.data,
        }
    }
}

pub struct BitboardIterator {
    remaining: u64,
}

impl Iterator for BitboardIterator {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            None
        } else {
            let index = self.remaining.trailing_zeros();
            self.remaining &= self.remaining - 1;
            Some(Square::from_u32(index))
        }
    }
}

impl Not for Bitboard {
    type Output = Bitboard;

    fn not(self) -> Self::Output {
        Self { data: !self.data }
    }
}

impl BitAnd for Bitboard {
    type Output = Bitboard;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            data: self.data & rhs.data,
        }
    }
}

impl BitOr for Bitboard {
    type Output = Bitboard;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            data: self.data | rhs.data,
        }
    }
}

impl BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.data |= rhs.data;
    }
}
