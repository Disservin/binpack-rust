mod common;
mod reader;
#[cfg(target_arch = "wasm32")]
mod wasm;
mod writer;

pub mod chess;

pub use common::binpack_error::BinpackError;
pub use common::entry::TrainingDataEntry;

pub use reader::CompressedReaderError;
pub use reader::CompressedTrainingDataEntryReader;
pub use reader::parse_chunk;
pub use reader::read_chunk_into;

pub use writer::CompressedTrainingDataEntryWriter;
pub use writer::CompressedWriterError;

#[cfg(target_arch = "wasm32")]
pub use wasm::parse_binpack_chunk;
