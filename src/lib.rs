#![no_std]
#![deny(unsafe_code)]
#![deny(missing_docs)]

//! A crate for reading RIFF-formatted data.
//!
//! Primarily designed for `no_std` targets, as no APIs from `std::io` are used
//! and an attempt is made to avoid needless copying to save RAM.

use core::convert::TryInto;
use core::fmt;
use core::str;
use core::u32;

/// A wrapper around an underlying RIFF-formatted byte slice,
/// which allows for reading Chunks from that pool.
#[derive(Debug, PartialEq, Clone)]
pub struct RiffReader<'a> {
    data: &'a [u8], // Underlying RIFF-encoded byte slice
    pos: usize,     // Index of next byte in the byte slice that should be read
}

/// RiffError is returned when invalid data is encountered or an end-of-underlying-data-pool is reached.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum RiffError {
    /// Non-ASCII ID was encountered at the given position in the underlying byte slice
    EncounteredInvalidIDNotASCII(usize),
    /// Non-ASCII ID was provided by the consumer
    InvalidIDNotASCII,
    /// Expected end of byte slice reached
    EndOfData,
    /// Unexpected end of byte slice reached (chunk length is greater than remaining number of bytes)
    UnexpectedEndOfData(usize, u32, usize),
}

impl fmt::Display for RiffError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use RiffError::*;
        match self {
            EncounteredInvalidIDNotASCII(pos) => write!(
                f,
                "ID at position {} in the underlying byte slice is not valid ASCII",
                pos
            ),
            InvalidIDNotASCII => write!(f, "Supplied ID is not valid ASCII"),
            EndOfData => write!(f, "End of the underlying byte slice reached"),
            UnexpectedEndOfData(len_pos, expected, have) => write!(f, "Expected {} bytes of data based on the index at position {}, however only {} are left", expected, len_pos, have),
        }
    }
}

/// A RIFF chunk.
#[derive(Debug, PartialEq, Clone, Copy)]
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
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ChunkId {
    id: [u8; 4],
}

impl ChunkId {
    /// Convert a 4 character str from ASCII to a `ChunkId`.
    /// Because it's not possible to elegantly enforce the length of a str through the type system,
    /// the string must be provided as a 4-element byte array representing ASCII characters instead.
    ///
    /// # Errors
    ///
    /// This function errors when not all bytes are valid ASCII characters.
    pub fn from_ascii(id: [u8; 4]) -> Result<ChunkId, RiffError> {
        if !id.is_ascii() {
            return Err(RiffError::InvalidIDNotASCII);
        }
        return Ok(ChunkId {
            // Never panics, as the length was checked beforehand
            id: id,
        });
    }

    /// Convert the `ChunkId` to a str.
    pub fn to_ascii(&self) -> &str {
        // Will never fail, as the consumer can't create a `ChunkId` with an invalid value
        return str::from_utf8(self.id.as_ref()).unwrap();
    }
}

impl RiffReader<'_> {
    /// Creates a new `RiffReader` over an underlying byte slice.
    ///
    /// Note that this function does not ensure that the underlying pool is valid RIFF data.
    pub fn new(data: &[u8]) -> RiffReader {
        return RiffReader { data: data, pos: 0 };
    }
    /// Reads the next chunk out of the underlying byte slice.
    ///
    /// Returns an error if the underlying byte slice is exhausted or invalid data is encountered.
    ///
    /// For efficiency reasons, the returned `Chunk` contains a reference to the data rather than a copy,
    /// meaning that it cannot live longer than the originating `RiffReader`.
    ///
    /// This may be turned into an iterator in the future, once the `Item` type of an `Iterator`
    /// can have an explicit lifetime.
    pub fn read_next_chunk(&mut self) -> Result<Chunk, RiffError> {
        let (new_pos, result) = read_chunk_at(self.data, self.pos);
        // Move to next chunk for next call
        self.pos = new_pos;

        return result;
    }

    /// Returns the chunk with the given ID, if present.
    /// If not present, returns `None`.
    /// Note that this does not recurse into chunks that can contain other chunks.
    pub fn get_chunk(&self, wanted_id: ChunkId) -> Option<Result<Chunk, RiffError>> {
        // TODO: Clean this up so we don't need a mutable reference
        // Iterate over each chunk until either a matching ID or end of data is encountered
        let mut pos: usize = 0;
        loop {
            let (new_pos, result) = read_chunk_at(self.data, pos);
            match result {
                Ok(chunk) => {
                    if chunk.id == wanted_id {
                        return Some(Ok(chunk));
                    }
                }
                Err(err) => match err {
                    // Exhausted without having found a matching chunk
                    RiffError::EndOfData => {
                        return None;
                    }
                    // Other errors are unexpected
                    _ => {
                        return Some(Err(err));
                    }
                },
            }
            pos = new_pos;
        }
    }
}

/// Read the chunk starting at byte pos, and also return the position of the next block.
fn read_chunk_at(data: &[u8], pos: usize) -> (usize, Result<Chunk, RiffError>) {
    // Roughly, a RIFF file consists of a bunch of chunks,
    // and each chunk consists of a 4 byte ID, 4 byte length to be interpreted as `u32`, and data following which is len bytes in size.

    // Therefore, at least 8 bytes have to be left in the underlying pool for a correct block to exist
    if (data.len() - pos) < 8 {
        return (pos, Err(RiffError::EndOfData));
    }
    // Read the ID
    let id_as_ascii: [u8; 4];
    let mut pos = pos;
    match read_id_at(data, pos) {
        (new_pos, Ok(val)) => {
            pos = new_pos;
            id_as_ascii = val;
        }
        (new_pos, Err(err)) => return (new_pos, Err(err)),
    };

    let id = ChunkId::from_ascii(id_as_ascii).unwrap();

    // Read the length
    let len: usize;
    match read_len_at(data, pos) {
        // TODO: Return error if conversion fails instead of panicking
        (new_pos, Ok(val)) => {
            pos = new_pos;
            len = val.try_into().unwrap();
        }
        (pos, Err(err)) => return (pos, Err(err)),
    }
    // Read len data bytes
    let payload_data: &[u8];
    match read_data_at(data, pos, len) {
        (new_pos, Ok(val)) => {
            pos = new_pos;
            payload_data = val;
        }
        (new_pos, Err(err)) => return (new_pos, Err(err)),
    }
    return (
        pos,
        Ok(Chunk {
            data: payload_data,
            id,
            len,
        }),
    );
}

fn read_id_at(data: &[u8], pos: usize) -> (usize, Result<[u8; 4], RiffError>) {
    let mut pos = pos;
    // Check whether we can actually read as much in order to prevent a runtime panic due to OOB index
    if (data.len() - pos) < 4 {
        return (pos, Err(RiffError::EndOfData));
    }
    let as_bytes = &data[pos..(pos + 4)];
    pos += 4;
    // ID must be valid ASCII
    if !as_bytes.is_ascii() {
        return (pos, Err(RiffError::EncounteredInvalidIDNotASCII(pos)));
    }
    // Will never panic, as length was checked above
    return (pos, Ok(as_bytes.try_into().unwrap()));
}

fn read_len_at(data: &[u8], pos: usize) -> (usize, Result<u32, RiffError>) {
    let mut pos = pos;
    // Check whether we can actually read as much in order to prevent a runtime panic due to OOB index
    if (data.len() - pos) < 4 {
        return (pos, Err(RiffError::EndOfData));
    }
    let len_as_bytes = &data[pos..(pos + 4)];
    // This panic will never happen, as we have obtained a subslice of length 4 in previous step
    let len = u32::from_le_bytes(len_as_bytes.try_into().unwrap());
    pos += 4;
    return (pos, Ok(len));
}

fn read_data_at(data: &[u8], pos: usize, len: usize) -> (usize, Result<&[u8], RiffError>) {
    let mut pos = pos;
    // Check whether remainder of backing byte slice is large enough
    if data.len() <= (pos + len) {
        let retval = Ok(&data[pos..(pos + len)]);
        pos += len;
        // Note that a padding byte is added if len is odd, meaning we have to advance the position by 1 extra.
        if (len % 2) != 0 {
            pos += 1;
        }
        return (pos, retval);
    }

    // If not, that means we encountered an unexpected end of data
    return (
        pos,
        Err(RiffError::UnexpectedEndOfData(
            pos - 4, // To get to starting index of length specifier
            len.try_into().unwrap(),
            data.len() - (pos + len),
        )),
    );
}
