use std::io::Write;

const HEADER_SIZE: usize = 8;

#[derive(Debug)]
struct Header {
    chunk_size: u32,
}

#[derive(Debug)]
pub struct CompressedTrainingDataFileWriter<T: Write> {
    file: T,
}

impl<T: Write> CompressedTrainingDataFileWriter<T> {
    pub fn new(file: T) -> std::io::Result<Self> {
        Ok(Self { file })
    }

    pub fn into_inner(self) -> std::io::Result<T> {
        Ok(self.file)
    }

    pub fn append(&mut self, data: &[u8]) -> std::io::Result<()> {
        let header = Header {
            chunk_size: data.len() as u32,
        };
        self.write_chunk_header(&header)?;
        self.file.write_all(data)?;
        Ok(())
    }

    fn write_chunk_header(&mut self, header: &Header) -> std::io::Result<()> {
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

    pub fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()
    }
}
