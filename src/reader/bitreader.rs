use std::rc::Rc;

#[derive(Debug)]
pub struct BitReader {
    // movetext: Rc<Vec<u8>>,
    data_ptr: *const u8,
    read_bits_left: usize,
    read_offset: usize,
    // base_offset: usize,
}

impl BitReader {
    pub fn new(movetext: Rc<Vec<u8>>, base_offset: usize) -> Self {
        let data_ptr = unsafe { movetext.as_ptr().add(base_offset) };

        Self {
            // movetext,
            data_ptr,
            read_bits_left: 8,
            read_offset: 0,
            // base_offset,
        }
    }

    pub fn extract_bits_le8(&mut self, count: usize) -> u8 {
        if count == 0 {
            return 0;
        }

        if self.read_bits_left == 0 {
            self.read_offset += 1;
            self.read_bits_left = 8;
        }

        unsafe {
            let byte = *self.data_ptr.add(self.read_offset) << (8 - self.read_bits_left);
            let mut bits = byte >> (8 - count);

            if count > self.read_bits_left {
                let spill_count = count - self.read_bits_left;
                bits |= *self.data_ptr.add(self.read_offset + 1) >> (8 - spill_count);
                self.read_bits_left += 8;
                self.read_offset += 1;
            }

            self.read_bits_left -= count;
            bits
        }
    }

    pub fn extract_vle16(&mut self, block_size: usize) -> u16 {
        let mask = (1 << block_size) - 1;
        let mut v = 0u16;
        let mut offset = 0;

        loop {
            let block = self.extract_bits_le8(block_size + 1) as u16;
            v |= (block & mask) << offset;
            if (block >> block_size) == 0 {
                break;
            }
            offset += block_size;
        }

        v
    }

    pub fn num_read_bytes(&self) -> usize {
        self.read_offset + (self.read_bits_left != 8) as usize
    }
}
