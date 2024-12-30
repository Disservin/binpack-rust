use std::io::{self};
use thiserror::Error;

use crate::{
    binpack_error::BinpackError, compressed_data_file::CompressedTrainingDataFile,
    data_entry::TrainingDataEntry, packed_entry::PackedTrainingDataEntry,
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

impl CompressedTrainingDataEntryReader {
    /// Create a new CompressedTrainingDataEntryReader,
    /// reading from the file at the given path.
    /// # Examples
    ///
    /// ```
    /// use sfbinpack::CompressedTrainingDataEntryReader;
    ///
    /// let mut reader = CompressedTrainingDataEntryReader::new("my.binpack").unwrap();
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
            input_file: CompressedTrainingDataFile::new(path, false)?,
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

        // Read packed entry
        let mut packed = PackedTrainingDataEntry::default();

        debug_assert!(
            self.offset + std::mem::size_of::<PackedTrainingDataEntry>() <= self.chunk.len()
        );

        packed.copy_from_slice(
            &self.chunk[self.offset..self.offset + std::mem::size_of::<PackedTrainingDataEntry>()],
        );

        self.offset += std::mem::size_of::<PackedTrainingDataEntry>();

        // Read number of plies
        let num_plies =
            ((self.chunk[self.offset] as u16) << 8) | (self.chunk[self.offset + 1] as u16);
        self.offset += 2;

        let entry = packed.unpack_entry();

        if num_plies > 0 {
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

    fn fetch_next_chunk_if_needed(&mut self) {
        if self.offset + std::mem::size_of::<PackedTrainingDataEntry>() + 2 > self.chunk.len() {
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
mod tests {}
