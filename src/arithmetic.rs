#[cfg(all(target_arch = "x86_64", target_feature = "bmi2"))]
use std::arch::x86_64::_pdep_u64;

#[cfg(all(target_arch = "x86_64", target_feature = "bmi2"))]
#[target_feature(enable = "bmi2")]
unsafe fn nth_set_bit_index_bmi2(v: u64, n: u64) -> u32 {
    _pdep_u64(1u64 << n, v).trailing_zeros() as u32
}

const fn nth_set_bit_index_naive(mut value: u64, n: usize) -> u8 {
    let mut count = 0;
    while count < n {
        if value == 0 {
            break;
        }
        value &= value - 1;
        count += 1;
    }
    value.trailing_zeros() as u8
}

const fn create_lookup_table() -> [[u8; 8]; 256] {
    let mut table = [[0u8; 8]; 256];
    let mut i = 0;
    while i < 256 {
        let mut j = 0;
        while j < 8 {
            table[i][j] = nth_set_bit_index_naive(i as u64, j);
            j += 1;
        }
        i += 1;
    }
    table
}

const NTH_SET_BIT_INDEX: [[u8; 8]; 256] = create_lookup_table();

#[allow(unreachable_code)]
#[inline]
pub fn nth_set_bit_index(v: u64, n: u64) -> u32 {
    #[cfg(all(target_arch = "x86_64", target_feature = "bmi2"))]
    unsafe {
        return nth_set_bit_index_bmi2(v, n);
    }

    let mut value = v;
    let mut count = n;

    let mut shift: u64 = 0;
    let p = (value & 0xFFFFFFFF).count_ones() as u64;
    let pmask = ((p > count) as u64).wrapping_sub(1);
    value >>= 32 & pmask;
    shift += 32 & pmask;
    count -= p & pmask;

    let p = (value & 0xFFFF).count_ones() as u64;
    let pmask = ((p > count) as u64).wrapping_sub(1);
    value >>= 16 & pmask;
    shift += 16 & pmask;
    count -= p & pmask;

    let p = (value & 0xFF).count_ones() as u64;
    let pmask = ((p > count) as u64).wrapping_sub(1);
    value >>= 8 & pmask;
    shift += 8 & pmask;
    count -= p & pmask;

    (NTH_SET_BIT_INDEX[(value & 0xFF) as usize][count as usize] as u64 + shift) as u32
}

pub fn unsigned_to_signed(r: u16) -> i16 {
    let mut v = (r << 15) | (r >> 1);
    if v & 0x8000 != 0 {
        v ^= 0x7FFF;
    }
    v as i16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nth_set_bit_index() {
        let test_value = 0b10110110u64;
        assert_eq!(nth_set_bit_index(test_value, 0), 1);
        assert_eq!(nth_set_bit_index(test_value, 1), 2);
        assert_eq!(nth_set_bit_index(test_value, 2), 4);
        assert_eq!(nth_set_bit_index(test_value, 3), 5);
        assert_eq!(nth_set_bit_index(test_value, 4), 7);
    }
}
