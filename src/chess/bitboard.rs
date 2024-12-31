use std::ops::{BitAnd, BitOr, BitOrAssign, Not};

use crate::chess::coords::{File, Rank, Square};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bitboard {
    data: u64,
}

impl Bitboard {
    pub const fn new(bits: u64) -> Self {
        Bitboard { data: bits }
    }

    /// Returns the number of set bits (popcount)
    #[must_use]
    #[inline(always)]
    pub fn count(&self) -> u32 {
        self.data.count_ones()
    }

    /// Returns the least significant bit
    #[inline(always)]
    pub fn lsb(&self) -> Square {
        Square::new(self.data.trailing_zeros())
    }

    /// Returns the most significant bit
    #[inline(always)]
    pub fn msb(&self) -> Square {
        Square::new(63 - self.data.leading_zeros())
    }

    /// Pops the least significant bit and returns it
    #[inline(always)]
    pub fn pop(&mut self) -> Square {
        let lsb = self.lsb();
        self.data &= self.data - 1;
        lsb
    }

    /// Toggle the bit on the given index, depending on the value (1 means set, 0 means clear)
    pub fn set(&mut self, index: u32, value: bool) {
        if value {
            self.data |= 1 << index;
        } else {
            self.data &= !(1 << index);
        }
    }

    /// Clear all bits
    pub fn clear(&mut self) {
        self.data = 0;
    }

    /// Checks if the bit on the given square is set
    pub fn is_set(&self, index: u32) -> bool {
        self.data & (1 << index) != 0
    }

    /// Set the bit on the given square
    pub fn sq_set(&self, index: Square) -> bool {
        self.data & (1 << index.index()) != 0
    }

    /// Create from a u64
    pub fn from_u64(data: u64) -> Self {
        Self { data }
    }

    /// Get the u64 representation
    pub fn bits(&self) -> u64 {
        self.data
    }

    pub fn from_before(index: u32) -> Self {
        Self {
            data: (1 << index) - 1,
        }
    }

    pub fn from_square(index: Square) -> Self {
        Self {
            data: 1 << index.index(),
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
        BitboardIterator { remaining: *self }
    }
}

pub struct BitboardIterator {
    remaining: Bitboard,
}

impl Iterator for BitboardIterator {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.bits() == 0 {
            None
        } else {
            Some(self.remaining.pop())
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

impl BitAnd<&Bitboard> for &Bitboard {
    type Output = Bitboard;
    fn bitand(self, rhs: &Bitboard) -> Bitboard {
        Bitboard {
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
