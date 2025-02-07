mod arithmetic;
mod binpack_error;
mod compressed_move;
mod compressed_position;
mod compressed_training_file;
mod entry;
mod reader;
mod writer;

pub mod chess;

pub use crate::binpack_error::BinpackError;

pub use crate::entry::TrainingDataEntry;

pub use crate::reader::CompressedReaderError;
pub use crate::reader::CompressedTrainingDataEntryReader;

pub use crate::writer::CompressedTrainingDataEntryWriter;
pub use crate::writer::CompressedWriterError;
