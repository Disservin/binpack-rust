use std::fmt;

use crate::chess::{position::Position, r#move::Move};

use super::{
    arithmetic::{signed_to_unsigned, unsigned_to_signed},
    compressed_move::CompressedMove,
    compressed_position::CompressedPosition,
};

/// A single training data entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrainingDataEntry {
    /// The position of the board.
    pub pos: Position,
    /// The which will be played on this position.
    pub mv: Move,
    /// The score of the position. Relative to the side to move of the current position.
    pub score: i16,
    /// The game ply of the position.
    pub ply: u16,
    /// The game result of the position.
    /// 1, 0, -1 for win, draw, loss for the side to move (like with score).
    pub result: i16,
}

impl TrainingDataEntry {
    pub fn is_continuation(&self, &other: &TrainingDataEntry) -> bool {
        self.result == -other.result
            && self.ply + 1 == other.ply
            && self.pos.after_move(self.mv) == other.pos
    }
}

impl fmt::Display for TrainingDataEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {}",
            self.pos.fen().unwrap(),
            self.mv.as_uci(),
            self.score,
            self.ply,
            self.result
        )
    }
}

#[derive(Debug, Default, Clone)]
pub struct PackedTrainingDataEntry {
    pub data: [u8; 32],
}

/// A packed training data entry.
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

    pub fn from_entry(entry: &TrainingDataEntry) -> Self {
        let mut packed = PackedTrainingDataEntry::default();
        let mut offset = 0;

        // Compress position
        // EBNF: Position
        let compressed_pos = CompressedPosition::compress(&entry.pos);
        compressed_pos.write_to_big_endian(&mut packed.data[offset..]);
        offset += CompressedPosition::byte_size();

        // Compress move
        // EBNF: Move
        let compressed_move = CompressedMove::compress(&entry.mv);
        compressed_move.write_to_big_endian(&mut packed.data[offset..]);
        offset += CompressedMove::byte_size();

        // Pack ply and result
        let pr = entry.ply | (signed_to_unsigned(entry.result) << 14);
        packed.data[offset] = (signed_to_unsigned(entry.score) >> 8) as u8;
        offset += 1;
        packed.data[offset] = signed_to_unsigned(entry.score) as u8;
        offset += 1;
        packed.data[offset] = (pr >> 8) as u8;
        offset += 1;
        packed.data[offset] = pr as u8;
        offset += 1;

        // Pack rule50 counter
        packed.data[offset] = (entry.pos.rule50_counter() >> 8) as u8;
        offset += 1;
        packed.data[offset] = entry.pos.rule50_counter() as u8;

        packed
    }

    fn read_u16_be(&self, offset: usize) -> u16 {
        ((self.data[offset] as u16) << 8) | (self.data[offset + 1] as u16)
    }
}

#[cfg(test)]
mod test {
    use crate::chess::{coords::Square, piece::Piece, r#move::MoveType};

    use super::*;

    #[test]
    fn test_packed_training_data_entry() {
        let data = [
            98, 121, 192, 21, 24, 76, 241, 100, 100, 106, 0, 4, 8, 48, 2, 17, 17, 145, 19, 117,
            247, 0, 0, 0, 61, 232, 0, 253, 0, 39, 0, 2,
        ];

        let packed_entry = PackedTrainingDataEntry::from_slice(&data);

        let entry = packed_entry.unpack_entry();

        let expected = TrainingDataEntry {
            pos: Position::from_fen(
                "1r3rk1/p2qnpb1/6pp/P1p1p3/3nN3/2QP2P1/R3PPBP/2B2RK1 b - - 2 20",
            )
            .unwrap(),
            mv: Move::new(
                Square::new(61),
                Square::new(58),
                MoveType::Normal,
                Piece::none(),
            ),
            score: -127,
            ply: 39,
            result: 0,
        };

        assert_eq!(entry, expected);
    }

    #[test]
    fn test_size_of_packed_training_data_entry() {
        assert_eq!(PackedTrainingDataEntry::byte_size(), 32);
    }
}
