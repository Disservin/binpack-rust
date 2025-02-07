#[derive(Debug)]
pub struct BitWriter {
    pub movetext: Vec<u8>,
    bits_left: usize,
}

impl BitWriter {
    pub fn new() -> Self {
        Self {
            movetext: Vec::new(),
            bits_left: 0,
        }
    }

    pub fn clear(&mut self) {
        self.movetext.clear();
        self.bits_left = 0;
    }

    pub fn add_bits_le8(&mut self, bits: u8, count: usize) {
        if count == 0 {
            return;
        }

        if self.bits_left == 0 {
            self.movetext.push(bits << (8 - count));
            self.bits_left = 8;
        } else if count <= self.bits_left {
            let last_idx = self.movetext.len() - 1;
            self.movetext[last_idx] |= bits << (self.bits_left - count);
        } else {
            let spill_count = count - self.bits_left;
            let last_idx = self.movetext.len() - 1;
            self.movetext[last_idx] |= bits >> spill_count;
            self.movetext.push(bits << (8 - spill_count));
            self.bits_left += 8;
        }

        self.bits_left -= count;
    }

    pub fn add_bits_vle16(&mut self, mut v: u16, block_size: usize) {
        let mask = (1 << block_size) - 1;
        loop {
            let block = ((v & mask) as u8) | (((v > mask) as u8) << block_size);
            self.add_bits_le8(block, block_size + 1);
            v >>= block_size;
            if v == 0 {
                break;
            }
        }
    }

    pub fn movetext(&self) -> &[u8] {
        &self.movetext
    }
}
