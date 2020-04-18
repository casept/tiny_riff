#![no_std]
#![deny(unsafe_code)]
#![deny(missing_docs)]

//! A crate for reading RIFF files.
//!
//! Primarily designed for `no_std` targets, as no APIs from `std::io` are used
//! and an attempt is made to avoid needless copying to save RAM.

use core::convert::TryInto;
use core::str;
use core::u32;

/// A wrapper around an underlying RIFF-formatted data pool,
/// which allows for reading Chunks from that pool.
#[derive(Debug)]
pub struct RiffReader<'a> {
    data: &'a [u8], // Underlying RIFF-encoded data pool
    pos: usize,     // Index of next byte in the data pool that should be read
}

/// RiffError is returned when invalid data is encountered or an end-of-underlying-data-pool is reached.
#[derive(Debug)]
pub struct RiffError {}

/// A RIFF chunk.
#[derive(Debug)]
pub struct Chunk<'a> {
    /// The actual payload data of the chunk
    pub data: &'a [u8],
    /// The ID of the chunk
    pub id: ChunkId,
    /// The length of the data in the chunk
    pub len: usize,
}

/// TODO: Implement turing Chunk into RiffReader for recursion-capable chunks

/// The ID of a RIFF chunk.
#[derive(Debug)]
pub struct ChunkId {
    id: [u8; 4],
}

impl ChunkId {
    /// Convert a 4 character str from ASCII to a `ChunkId`.
    /// Note that the str must be exactly 4 ASCII characters.
    ///
    /// # Errors
    ///
    /// This function errors when the str is not 4 characters long, or not all characters are valid ASCII.
    pub fn from_ascii(id: &str) -> Result<ChunkId, RiffError> {
        if id.len() != 4 {
            return Err(RiffError {});
        }
        if !id.is_ascii() {
            return Err(RiffError {});
        }
        return Ok(ChunkId {
            // Never panics, as the length was checked beforehand
            id: id.as_bytes().try_into().unwrap(),
        });
    }

    /// Convert the `ChunkId` to a str.
    pub fn to_ascii(&self) -> &str {
        // Will never fail, as the consumer can't create a `ChunkId` with an invalid value
        return str::from_utf8(self.id.as_ref()).unwrap();
    }
}

impl RiffReader<'_> {
    /// Creates a new `RiffReader` over an underlying data pool.
    ///
    /// Note that this function does not ensure that the underlying pool is valid RIFF data.
    pub fn new(data: &[u8]) -> RiffReader {
        return RiffReader { data: data, pos: 0 };
    }
    /// Reads the next chunk out of the underlying data pool.
    ///
    /// Returns an error if the underlying data pool is exhausted or invalid data is encountered.
    ///
    /// For efficiency reasons, the returned `Chunk` contains a reference to the data rather than a copy,
    /// meaning that it cannot live longer than the originating `RiffReader`.
    pub fn read_next_chunk(&mut self) -> Result<Chunk, RiffError> {
        // Roughly, a RIFF file consists of a bunch of chunks,
        // and each chunk consists of a 4 byte ID, 4 byte length to be interpreted as `u32`, and data following which is len bytes in size.

        // Therefore, at least 8 bytes have to be left in the underlying pool for a correct block to exist
        if (self.data.len() - self.pos) < 8 {
            return Err(RiffError {});
        }
        // Read the ID
        let id = ChunkId::from_ascii(self.read_next_id()?)?;
        // Read the length
        // TODO: Return error if conversion fails instead of panicking
        let len: usize = self.read_next_len()?.try_into().unwrap();
        // Read len data bytes
        let data = self.read_next_data(len)?;
        return Ok(Chunk { data, id, len });
    }

    fn read_next_id(&mut self) -> Result<&str, RiffError> {
        // Check whether we can actually read as much in order to prevent a runtime panic due to OOB index
        if (self.data.len() - self.pos) < 4 {
            return Err(RiffError {});
        }
        let as_bytes = &self.data[self.pos..(self.pos + 4)];
        self.pos = self.pos + 4;
        // ID must be valid ASCII
        if !as_bytes.is_ascii() {
            return Err(RiffError {});
        }
        // Will never panic, as UTF-8 is strict superset of ASCII
        return Ok(str::from_utf8(as_bytes).unwrap());
    }

    fn read_next_len(&mut self) -> Result<u32, RiffError> {
        // Check whether we can actually read as much in order to prevent a runtime panic due to OOB index
        if (self.data.len() - self.pos) < 4 {
            return Err(RiffError {});
        }
        let len_as_bytes = &self.data[self.pos..(self.pos + 4)];
        // This panic will never happen, as we have obtained a subslice of length 4 in previous step
        let len = u32::from_le_bytes(len_as_bytes.try_into().unwrap());
        return Ok(len);
    }

    fn read_next_data(&mut self, len: usize) -> Result<&[u8], RiffError> {
        // Check whether remainder of backing data pool is large enough
        if self.data.len() <= (self.pos + len) {
            // Note that a padding byte is added if len is odd, meaning we have to advance the position by 1 extra.
            if (len % 2) != 0 {
                self.pos = self.pos + len + 1;
            } else {
                self.pos = self.pos + len;
            }
            return Ok(&self.data[self.pos..(self.pos + len)]);
        }

        return Err(RiffError {});
    }

    /// Returns the chunk with the given ID, if present.
    /// If not present, returns `None`.
    /// Note that this does not recurse into chunks that can contain other chunks.
    pub fn read_chunk(&self, chunk: ChunkId) -> Option<Result<Chunk, RiffError>> {
        // TODO: Implement
        return None;
    }
}

// TODO: impl IntoIterator for RiffReader
