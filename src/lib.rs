mod common;
mod reader;
mod writer;

pub mod chess;

pub use common::binpack_error::BinpackError;
pub use common::entry::TrainingDataEntry;

pub use reader::CompressedReaderError;
pub use reader::CompressedTrainingDataEntryReader;

pub use writer::CompressedTrainingDataEntryWriter;
pub use writer::CompressedWriterError;
