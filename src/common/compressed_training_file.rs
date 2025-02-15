use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};

use super::binpack_error::{BinpackError, Result};

const HEADER_SIZE: usize = 8;

const KI_B: u32 = 1024;
const MI_B: u32 = 1024 * KI_B;

const MAX_CHUNK_SIZE: u32 = 100 * MI_B;

/// Magic bytes that identify the chess binary format
const MAGIC: &[u8; 4] = b"BINP";

#[derive(Debug)]
struct Header {
    chunk_size: u32,
}

#[derive(Debug)]
pub struct CompressedTrainingDataFile {
    file: File,
    read_bytes: u64,
}

impl CompressedTrainingDataFile {
    pub fn new(path: &str, append: bool, create: bool) -> io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(create)
            .append(append)
            .open(path)?;

        Ok(Self {
            file,
            read_bytes: 0,
        })
    }

    pub fn append(&mut self, data: &[u8]) -> io::Result<()> {
        let header = Header {
            chunk_size: data.len() as u32,
        };
        self.write_chunk_header(&header)?;
        self.file.write_all(data)?;
        Ok(())
    }

    pub fn read_bytes(&self) -> u64 {
        self.read_bytes
    }

    pub fn has_next_chunk(&mut self) -> bool {
        if let Ok(pos) = self.file.stream_position() {
            if let Ok(len) = self.file.seek(SeekFrom::End(0)) {
                if self.file.seek(SeekFrom::Start(pos)).is_ok() {
                    return pos < len;
                }
            }
        }
        false
    }

    /// EBNF: BLOCK
    pub fn read_next_chunk(&mut self) -> Result<Vec<u8>> {
        // EBNF: ChunkHeader
        let header = self.read_chunk_header()?;

        let mut data = vec![0u8; (header.chunk_size) as usize];

        // EBNF: Chain
        self.file.read_exact(&mut data)?;

        self.read_bytes += header.chunk_size as u64;

        Ok(data)
    }

    fn write_chunk_header(&mut self, header: &Header) -> io::Result<()> {
        let mut buf = [0u8; HEADER_SIZE];
        buf[0] = b'B';
        buf[1] = b'I';
        buf[2] = b'N';
        buf[3] = b'P';
        buf[4] = (header.chunk_size & 0xFF) as u8;
        buf[5] = ((header.chunk_size >> 8) & 0xFF) as u8;
        buf[6] = ((header.chunk_size >> 16) & 0xFF) as u8;
        buf[7] = ((header.chunk_size >> 24) & 0xFF) as u8;
        self.file.write_all(&buf)
    }

    // Reads the block and chunksize
    fn read_chunk_header(&mut self) -> Result<Header> {
        let mut buf = [0u8; HEADER_SIZE];

        match self.file.read_exact(&mut buf) {
            Ok(_) => (),
            Err(_) => return Err(BinpackError::InvalidMagic),
        }

        self.read_bytes += HEADER_SIZE as u64;

        if &buf[0..4] != MAGIC {
            return Err(BinpackError::InvalidMagic);
        }

        let chunk_size = u32::from_le_bytes(buf[4..8].try_into().unwrap());

        if chunk_size > MAX_CHUNK_SIZE {
            return Err(BinpackError::InvalidFormat(
                "Chunk size larger than supported. Malformed file?".to_string(),
            ));
        }

        Ok(Header { chunk_size })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use io::Write;
    use tempfile::NamedTempFile;

    fn create_test_file(data: &[u8]) -> NamedTempFile {
        let mut file = tempfile::NamedTempFile::new().unwrap();

        // Manually write header
        let mut header = [0u8; HEADER_SIZE];
        header[0] = b'B';
        header[1] = b'I';
        header[2] = b'N';
        header[3] = b'P';
        header[4..8].copy_from_slice(&(data.len() as u32).to_le_bytes());

        file.write_all(&header).unwrap();
        file.write_all(data).unwrap();
        file.flush().unwrap();

        file
    }

    #[test]
    fn test_new_file_creation() {
        let temp_path = NamedTempFile::new().unwrap();
        let file =
            CompressedTrainingDataFile::new(temp_path.path().to_str().unwrap(), false, false);
        assert!(file.is_ok());
    }

    #[test]
    fn test_read_valid_chunk() {
        let test_data = b"Hello, World!";
        let temp_file = create_test_file(test_data);

        let mut file =
            CompressedTrainingDataFile::new(temp_file.path().to_str().unwrap(), false, false)
                .unwrap();
        assert!(file.has_next_chunk());

        let chunk = file.read_next_chunk().unwrap();
        assert_eq!(chunk, test_data);
        assert_eq!(file.read_bytes(), (HEADER_SIZE + test_data.len()) as u64);
    }

    #[test]
    fn test_has_next_chunk() {
        let test_data = b"Test Data";
        let temp_file = create_test_file(test_data);

        let mut file =
            CompressedTrainingDataFile::new(temp_file.path().to_str().unwrap(), false, false)
                .unwrap();
        assert!(file.has_next_chunk());

        let _ = file.read_next_chunk().unwrap();
        assert!(!file.has_next_chunk());
    }

    #[test]
    fn test_invalid_magic_number() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"INVALID").unwrap();

        let mut file =
            CompressedTrainingDataFile::new(temp_file.path().to_str().unwrap(), false, false)
                .unwrap();
        match file.read_next_chunk() {
            Err(BinpackError::InvalidMagic) => (),
            _ => panic!("Expected InvalidMagic error"),
        }
    }

    #[test]
    fn test_chunk_size_too_large() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let mut header = [0u8; HEADER_SIZE];
        header[0..4].copy_from_slice(b"BINP");
        header[4..8].copy_from_slice(&(MAX_CHUNK_SIZE + 1).to_le_bytes());
        temp_file.write_all(&header).unwrap();

        let mut file =
            CompressedTrainingDataFile::new(temp_file.path().to_str().unwrap(), false, false)
                .unwrap();
        match file.read_next_chunk() {
            Err(BinpackError::InvalidFormat(_)) => (),
            _ => panic!("Expected InvalidFormat error"),
        }
    }

    #[test]
    fn test_multiple_chunks() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let chunks = vec![b"Chunk1", b"Chunk2", b"Chunk3"];

        // Write multiple chunks
        for chunk in &chunks {
            let mut header = [0u8; HEADER_SIZE];
            header[0..4].copy_from_slice(b"BINP");
            header[4..8].copy_from_slice(&(chunk.len() as u32).to_le_bytes());
            temp_file.write_all(&header).unwrap();
            temp_file.write_all(*chunk).unwrap();
        }
        temp_file.flush().unwrap();

        let mut file =
            CompressedTrainingDataFile::new(temp_file.path().to_str().unwrap(), false, false)
                .unwrap();

        for expected_chunk in chunks {
            assert!(file.has_next_chunk());
            let chunk = file.read_next_chunk().unwrap();
            assert_eq!(chunk, expected_chunk);
        }

        assert!(!file.has_next_chunk());
    }
}
