use std::io::Write;
use std::io::{self};
use thiserror::Error;

use crate::{
    chess::{position::Position, r#move::Move},
    common::{
        compressed_training_file_writer::CompressedTrainingDataFileWriter,
        entry::PackedTrainingDataEntry, entry::TrainingDataEntry,
    },
};

use super::move_score_list::PackedMoveScoreList;

const KI_B: usize = 1024;
const MI_B: usize = 1024 * KI_B;

const SUGGESTED_CHUNK_SIZE: usize = MI_B;
const MAX_MOVELIST_SIZE: usize = 10 * KI_B;

#[derive(Debug, Error)]
pub enum CompressedWriterError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Invalid data format: {0}")]
    InvalidFormat(String),
    #[error("End of file reached")]
    EndOfFile,
}

type Result<T> = std::result::Result<T, CompressedWriterError>;

/// Write Stockfish binpacks from TrainingDataEntry's
/// to a file.
#[derive(Debug)]
pub struct CompressedTrainingDataEntryWriter<T: Write> {
    output_file: Option<CompressedTrainingDataFileWriter<T>>,
    last_entry: TrainingDataEntry,
    movelist: PackedMoveScoreList,
    packed_size: usize,
    packed_entries: Vec<u8>,
    is_first: bool,
}

impl<T: Write> CompressedTrainingDataEntryWriter<T> {
    /// Create a new CompressedTrainingDataEntryWriter,
    /// writing to the file at the given path.
    /// The file will only be completely saved when the writer is dropped!
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fs::File;
    /// use sfbinpack::CompressedTrainingDataEntryWriter;
    ///
    /// let file = File::options().read(true).write(true).create(true).open("test/ep1.binpack").unwrap();
    /// let mut writer = CompressedTrainingDataEntryWriter::new(file).unwrap();
    /// ```
    pub fn new(file: T) -> Result<Self> {
        let writer = Self {
            output_file: Some(CompressedTrainingDataFileWriter::new(file)?),
            last_entry: TrainingDataEntry {
                ply: 0xFFFF, // never a continuation
                result: 0x7FFF,
                pos: Position::default(),
                mv: Move::default(),
                score: 0,
            },
            movelist: PackedMoveScoreList::new(),
            packed_size: 0,
            packed_entries: vec![0u8; SUGGESTED_CHUNK_SIZE + MAX_MOVELIST_SIZE],
            is_first: true,
        };
        Ok(writer)
    }

    pub fn into_inner(&mut self) -> io::Result<T> {
        self.output_file.take().unwrap().into_inner()
    }

    /// Write a single entry to the file
    pub fn write_entry(&mut self, entry: &TrainingDataEntry) -> Result<()> {
        let is_cont = self.last_entry.is_continuation(entry);

        if is_cont {
            self.movelist
                .add_move_score(&entry.pos, entry.mv, entry.score);
        } else {
            if !self.is_first {
                self.write_movelist();
            }

            if self.packed_size >= SUGGESTED_CHUNK_SIZE {
                match self
                    .output_file
                    .as_mut()
                    .unwrap()
                    .append(&self.packed_entries[..self.packed_size])
                {
                    Ok(_) => {}
                    Err(e) => {
                        return Err(CompressedWriterError::Io(e));
                    }
                }
                self.packed_size = 0;
            }

            let packed = PackedTrainingDataEntry::from_entry(entry);
            let packed_bytes: [u8; size_of::<PackedTrainingDataEntry>()] = packed.data;

            self.packed_entries
                [self.packed_size..self.packed_size + PackedTrainingDataEntry::byte_size()]
                .copy_from_slice(&packed_bytes);

            self.packed_size += PackedTrainingDataEntry::byte_size();

            self.movelist.clear(entry);
            self.is_first = false;
        }

        self.last_entry = *entry;
        Ok(())
    }

    pub fn flush(&mut self) {
        self.output_file.as_mut().unwrap().flush().unwrap();
    }

    /// Flush the buffer to the file, automatically called when the writer is dropped
    fn flush_packed(&mut self) -> Result<()> {
        if self.packed_size > 0 {
            if !self.is_first {
                self.write_movelist();
            }

            match self
                .output_file
                .as_mut()
                .unwrap()
                .append(&self.packed_entries[..self.packed_size])
            {
                Ok(_) => {}
                Err(e) => {
                    return Err(CompressedWriterError::Io(e));
                }
            }
            self.packed_size = 0;
        }

        self.output_file.as_mut().unwrap().flush()?;

        Ok(())
    }

    fn write_movelist(&mut self) {
        self.packed_entries[self.packed_size] = (self.movelist.num_plies >> 8) as u8;
        self.packed_entries[self.packed_size + 1] = self.movelist.num_plies as u8;
        self.packed_size += 2;

        if self.movelist.num_plies > 0 {
            let movetext = self.movelist.movetext();
            self.packed_entries[self.packed_size..self.packed_size + movetext.len()]
                .copy_from_slice(movetext);
            self.packed_size += movetext.len();
        }
    }
}

impl<T: Write> Drop for CompressedTrainingDataEntryWriter<T> {
    fn drop(&mut self) {
        if let Err(e) = self.flush_packed() {
            eprintln!("Error flushing writer: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{self, OpenOptions},
        io::{Cursor, Read, Seek},
    };

    use super::*;

    use crate::chess::{
        coords::Square,
        piece::Piece,
        position::Position,
        r#move::{Move, MoveType},
    };

    #[test]
    fn test_compressed_writer() {
        let entries = vec![
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

        {
            // delete file
            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .append(false)
                .open("test/ep_new1.binpack")
                .unwrap();

            let mut writer = CompressedTrainingDataEntryWriter::new(file).unwrap();

            for entry in entries.iter() {
                writer.write_entry(entry).unwrap();
            }
        }

        // check that ep_new1.binpack equals ep1.binpack
        let file1_bytes = fs::read("test/ep_new1.binpack").unwrap();
        let file2_bytes = fs::read("test/ep1.binpack").unwrap();

        assert_eq!(file1_bytes, file2_bytes);

        let _ = fs::remove_file("test/ep_new1.binpack");
    }

    #[test]
    fn test_compressed_writer_in_memory_file() {
        let entries = vec![
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

        let cursor = Cursor::new(Vec::new());
        let mut writer = CompressedTrainingDataEntryWriter::new(cursor).unwrap();

        for entry in entries.iter() {
            writer.write_entry(entry).unwrap();
        }

        writer.flush();

        let mut cursor = writer.into_inner().unwrap();
        cursor.seek(io::SeekFrom::Start(0)).unwrap();

        let mut read_bytes = vec![];
        cursor.read_to_end(&mut read_bytes).unwrap();

        let expected_bytes = fs::read("test/ep1.binpack").unwrap();
        assert_eq!(read_bytes, expected_bytes);
    }

    #[test]
    fn test_compressed_writer_big_score_diff() {
        let entries = vec![
            TrainingDataEntry {
                pos: Position::from_fen("1q5b/1r5k/4p2p/1b2P1pN/3p4/6PP/1nP3B1/1Q2B1K1 w - - 0 35")
                    .unwrap(),
                mv: Move::new(
                    Square::new(10),
                    Square::new(26),
                    MoveType::Normal,
                    Piece::none(),
                ),
                score: -31999,
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
                score: -1500,
                ply: 69,
                result: 0,
            },
        ];

        let cursor = Cursor::new(Vec::new());
        let mut writer = CompressedTrainingDataEntryWriter::new(cursor).unwrap();

        for entry in entries.iter() {
            writer.write_entry(entry).unwrap();
        }

        writer.flush();

        let mut cursor = writer.into_inner().unwrap();
        cursor.seek(io::SeekFrom::Start(0)).unwrap();

        let mut read_bytes = vec![];
        cursor.read_to_end(&mut read_bytes).unwrap();

        let expected_bytes = [
            66, 73, 78, 80, 37, 0, 0, 0, 130, 130, 144, 210, 8, 192, 70, 82, 72, 58, 64, 0, 81, 16,
            18, 113, 155, 5, 0, 0, 0, 0, 0, 0, 10, 104, 249, 253, 0, 68, 0, 0, 0, 1, 29, 83, 79,
        ];
        assert_eq!(read_bytes, expected_bytes);
    }
}
