use std::io::{Read, Seek, SeekFrom};

use super::binpack_error::{BinpackError, Result};

const HEADER_SIZE: usize = 8;
const MAX_CHUNK_SIZE: u32 = 100 * 1024 * 1024;
const MAGIC: &[u8; 4] = b"BINP";

#[derive(Debug)]
struct Header {
    chunk_size: u32,
}

#[derive(Debug)]
pub struct CompressedTrainingDataFileReader<T: Read + Seek> {
    file: T,
    read_bytes: u64,
}

impl<T: Read + Seek> CompressedTrainingDataFileReader<T> {
    pub fn new(file: T) -> std::io::Result<Self> {
        Ok(Self {
            file,
            read_bytes: 0,
        })
    }

    pub fn into_inner(self) -> std::io::Result<T> {
        Ok(self.file)
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

    pub fn read_next_chunk(&mut self) -> Result<Vec<u8>> {
        let header = self.read_chunk_header()?;
        let mut data = vec![0u8; header.chunk_size as usize];
        self.file.read_exact(&mut data)?;
        self.read_bytes += header.chunk_size as u64;
        Ok(data)
    }

    pub fn read_next_chunk_into(&mut self, buffer: &mut Vec<u8>) -> Result<()> {
        let header = self.read_chunk_header()?;
        buffer.resize(header.chunk_size as usize, 0);
        self.file.read_exact(buffer)?;
        self.read_bytes += header.chunk_size as u64;
        Ok(())
    }

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
