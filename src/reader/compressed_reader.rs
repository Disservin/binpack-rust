use std::io::{self};
use thiserror::Error;

use crate::common::{
    binpack_error::BinpackError, compressed_training_file::CompressedTrainingDataFile,
    entry::PackedTrainingDataEntry, entry::TrainingDataEntry,
};

use super::move_score_list_reader::PackedMoveScoreListReader;

const SUGGESTED_CHUNK_SIZE: usize = 8192;

#[derive(Debug, Error)]
pub enum CompressedReaderError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Invalid data format: {0}")]
    InvalidFormat(String),
    #[error("End of file reached")]
    EndOfFile,
    #[error("Binpack error: {0}")]
    BinpackError(#[from] BinpackError),
}

type Result<T> = std::result::Result<T, CompressedReaderError>;

/// Reads Stockfish binpacks and returns a TrainingDataEntry
/// for each encoded entry.
#[derive(Debug)]
pub struct CompressedTrainingDataEntryReader {
    chunk: Vec<u8>,
    movelist_reader: Option<OwnedMoveScoreListReader>,
    input_file: CompressedTrainingDataFile,
    offset: usize,
    file_size: u64,
    is_end: bool,
}

#[derive(Debug)]
struct OwnedMoveScoreListReader {
    reader: PackedMoveScoreListReader<'static>,
}

/*
Search for EBNF: ..., to find the implementation.

File         = Block*
Block        = ChunkHeader Chain*
ChunkHeader  = Magic ChunkSize
Magic        = "BINP"
ChunkSize    = UINT32LE               (* 4 bytes, little endian *)

Chain        = Stem Count MoveText
Stem         = Position Move Score PlyResult Rule50
Count        = UINT16BE               (* 2 bytes, big endian *)
MoveText     = MoveScore*

(* Stem components - total 32 bytes *)
Position     = CompressedPosition     (* 24 bytes *)
Move         = CompressedMove         (* 2 bytes *)
Score        = INT16BE                (* 2 bytes, big endian, signed *)
PlyResult    = UINT8                  (* 2 byte, big endian unsigned *)
Rule50       = UINT16BE               (* 2 bytes, big endian *)

(* MoveText components *)
MoveScore    = EncodedMove EncodedScore

(* Encoded components *)
EncodedMove  = VARLEN_UINT            (* Variable length encoding *)
EncodedScore = VARLEN_INT             (* Variable length encoding *)
*/

// EBNF: File
impl CompressedTrainingDataEntryReader {
    /// Create a new CompressedTrainingDataEntryReader,
    /// reading from the file at the given path.
    /// # Examples
    ///
    /// ```
    /// use sfbinpack::CompressedTrainingDataEntryReader;
    ///
    /// let mut reader = CompressedTrainingDataEntryReader::new("test/ep1.binpack").unwrap();
    ///
    /// while reader.has_next() {
    ///     let entry = reader.next();
    /// }
    /// ```
    pub fn new(path: &str) -> Result<Self> {
        let chunk = Vec::with_capacity(SUGGESTED_CHUNK_SIZE);

        let mut reader = Self {
            chunk,
            movelist_reader: None,
            input_file: CompressedTrainingDataFile::new(path, false, false)?,
            offset: 0,
            file_size: std::fs::metadata(path)?.len(),
            is_end: false,
        };

        if !reader.input_file.has_next_chunk() {
            reader.is_end = true;
            return Err(CompressedReaderError::EndOfFile);
        } else {
            reader.chunk = match reader.input_file.read_next_chunk() {
                Ok(chunk) => chunk,
                Err(e) => return Err(CompressedReaderError::BinpackError(e)),
            };
        }

        Ok(reader)
    }

    /// Get the size of the file in bytes
    pub fn file_size(&self) -> u64 {
        self.file_size
    }

    /// Get how much of the file has been read so far
    pub fn read_bytes(&self) -> u64 {
        self.input_file.read_bytes()
    }

    /// Check if there are more TrainingDataEntry to read
    pub fn has_next(&self) -> bool {
        !self.is_end
    }

    /// Check if the next entry is a continuation of the last returned entry from next()
    pub fn is_next_entry_continuation(&self) -> bool {
        if let Some(ref reader) = self.movelist_reader {
            return reader.reader.has_next();
        }

        false
    }

    /// Get the next TrainingDataEntry
    pub fn next(&mut self) -> TrainingDataEntry {
        if let Some(ref mut reader) = self.movelist_reader {
            let entry = reader.reader.next_entry();

            if !reader.reader.has_next() {
                self.offset += reader.reader.num_read_bytes();
                self.movelist_reader = None;
                self.fetch_next_chunk_if_needed();
            }

            return entry;
        }

        // We don't have a movelist reader, so we first need to extract the "stem" information

        // EBNF: Stem
        let entry = self.read_entry();

        // EBNF: Count
        let num_plies = self.read_plies();

        if num_plies > 0 {
            // EBNF: MoveText
            let chunk_ref = &self.chunk[self.offset..];

            // should be safe lol, someone rewrite this please
            let reader = unsafe {
                std::mem::transmute::<
                    PackedMoveScoreListReader<'_>,
                    PackedMoveScoreListReader<'static>,
                >(PackedMoveScoreListReader::new(entry, chunk_ref, num_plies))
            };

            self.movelist_reader = Some(OwnedMoveScoreListReader { reader });
        } else {
            self.fetch_next_chunk_if_needed();
        }

        entry
    }

    fn read_entry(&mut self) -> TrainingDataEntry {
        let size = PackedTrainingDataEntry::byte_size();

        debug_assert!(self.offset + size <= self.chunk.len());

        let packed =
            PackedTrainingDataEntry::from_slice(&self.chunk[self.offset..self.offset + size]);

        self.offset += size;

        packed.unpack_entry()
    }

    fn read_plies(&mut self) -> u16 {
        let ply = ((self.chunk[self.offset] as u16) << 8) | (self.chunk[self.offset + 1] as u16);
        self.offset += 2;
        ply
    }

    // EBNF: BLOCK
    fn fetch_next_chunk_if_needed(&mut self) {
        if self.offset + PackedTrainingDataEntry::byte_size() + 2 > self.chunk.len() {
            if self.input_file.has_next_chunk() {
                let chunk = self.input_file.read_next_chunk().unwrap();
                self.chunk = chunk;
                self.offset = 0;
            } else {
                self.is_end = true;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::chess::{
        coords::Square,
        piece::Piece,
        position::Position,
        r#move::{Move, MoveType},
    };

    use super::*;

    #[test]
    fn test_reader_simple() {
        let mut reader = CompressedTrainingDataEntryReader::new("./test/ep1.binpack").unwrap();

        let mut entries: Vec<TrainingDataEntry> = Vec::new();

        while reader.has_next() {
            let entry = reader.next();

            entries.push(entry);
        }

        let expected = vec![
            TrainingDataEntry {
                pos: Position::from_fen("1q5b/1r5k/4p2p/1b2P1pN/3p4/6PP/1nP3B1/1Q2B1K1 w - - 0 35")
                    .unwrap(),
                mv: Move::new(
                    Square::new(10),
                    Square::new(26),
                    MoveType::Normal,
                    Piece::none(),
                ),
                score: -201,
                ply: 68,
                result: 0,
            },
            TrainingDataEntry {
                pos: Position::from_fen("1q5b/1r5k/4p2p/1b2P1pN/2Pp4/6PP/1n4B1/1Q2B1K1 b - - 0 35")
                    .unwrap(),
                mv: Move::new(
                    Square::new(27),
                    Square::new(19),
                    MoveType::Normal,
                    Piece::none(),
                ),
                score: 254,
                ply: 69,
                result: 0,
            },
            TrainingDataEntry {
                pos: Position::from_fen(
                    "1q5b/1r5k/4p2p/1b2P1pN/2P5/3p2PP/1n4B1/1Q2B1K1 w - - 0 36",
                )
                .unwrap(),
                mv: Move::new(
                    Square::new(14),
                    Square::new(49),
                    MoveType::Normal,
                    Piece::none(),
                ),
                score: -220,
                ply: 70,
                result: 0,
            },
        ];

        assert_eq!(entries, expected);
    }
}
