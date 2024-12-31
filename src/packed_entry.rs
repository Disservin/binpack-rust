use crate::{
    arithmetic::unsigned_to_signed, compressed_move::CompressedMove,
    compressed_position::CompressedPosition, data_entry::TrainingDataEntry,
};

#[derive(Debug, Default, Clone)]
pub struct PackedTrainingDataEntry {
    pub data: [u8; 32],
}

impl PackedTrainingDataEntry {
    pub fn from_slice(slice: &[u8]) -> Self {
        PackedTrainingDataEntry {
            data: slice.try_into().unwrap(),
        }
    }

    pub fn byte_size() -> usize {
        std::mem::size_of::<PackedTrainingDataEntry>()
    }

    pub fn unpack_entry(&self) -> TrainingDataEntry {
        let mut offset = 0;

        // Read and decompress position
        // EBNF: Position
        let compressed_pos = CompressedPosition::read_from_big_endian(&self.data[offset..]);
        let mut pos = compressed_pos.decompress();
        offset += CompressedPosition::byte_size();

        // Read and decompress move
        // EBNF: Move
        let compressed_move = CompressedMove::read_from_big_endian(&self.data[offset..]);
        let mv = compressed_move.decompress();
        offset += CompressedMove::byte_size();

        // Read score
        // EBNF: Score
        let score = unsigned_to_signed(self.read_u16_be(offset));
        offset += 2;

        // Read ply and result (packed together)
        // EBNF: PlyResult
        let pr = self.read_u16_be(offset);
        let ply = pr & 0x3FFF;
        let result = unsigned_to_signed(pr >> 14);
        offset += 2;

        // Set position's ply
        pos.set_ply(ply);

        // Read and set rule50 counter
        // EBNF: Rule50
        pos.set_rule50_counter(self.read_u16_be(offset));

        TrainingDataEntry {
            pos,
            mv,
            score,
            ply,
            result,
        }
    }

    fn read_u16_be(&self, offset: usize) -> u16 {
        ((self.data[offset] as u16) << 8) | (self.data[offset + 1] as u16)
    }
}
