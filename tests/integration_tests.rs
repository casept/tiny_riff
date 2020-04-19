extern crate tiny_riff;

use tiny_riff::*;

use std::fs::File;
use std::io::Read;

static MINIMAL_DATA: &[u8] = &[
    0x73, 0x6D, 0x70, 0x6C, 0x74, 0x65, 0x73, 0x74, 0x01, 0x00, 0x00, 0x00, 0xFF, 0x00,
];
static MINIMAL_CHUNK_ID: [u8; 4] = [0x52, 0x49, 0x46, 0x46];
static MINIMAL_DATA_LEN: usize = 0x0E;

#[test]
fn read_minimal_next_chunk() {
    let mut file = File::open("test_assets/minimal.riff").unwrap();
    let mut test_data = Vec::new();
    file.read_to_end(&mut test_data).unwrap();
    let mut riff_reader = RiffReader::new(test_data.as_ref());
    let chunk = riff_reader.read_next_chunk().unwrap();
    assert_eq!(chunk.len, MINIMAL_DATA_LEN);
    assert_eq!(chunk.id, ChunkId::from_ascii(MINIMAL_CHUNK_ID).unwrap());
    assert_eq!(chunk.data, MINIMAL_DATA);
}

#[test]
fn read_minimal_get_by_chunk_name() {
    let mut file = File::open("test_assets/minimal.riff").unwrap();
    let mut test_data = Vec::new();
    file.read_to_end(&mut test_data).unwrap();
    let riff_reader = RiffReader::new(test_data.as_ref());
    let expected_id = ChunkId::from_ascii(MINIMAL_CHUNK_ID).unwrap();
    let chunk = riff_reader.get_chunk(expected_id).unwrap().unwrap();
    assert_eq!(chunk.len, MINIMAL_DATA_LEN);
    assert_eq!(chunk.id, expected_id);
    assert_eq!(chunk.data, MINIMAL_DATA);
}

#[test]
fn read_minimal_get_nonexistent_chunk() {
    let mut file = File::open("test_assets/minimal.riff").unwrap();
    let mut test_data = Vec::new();
    file.read_to_end(&mut test_data).unwrap();
    let riff_reader = RiffReader::new(test_data.as_ref());
    let nonexistent_id = ChunkId::from_ascii([0x41, 0x41, 0x41, 0x41]).unwrap();
    let chunk = riff_reader.get_chunk(nonexistent_id);
    assert_eq!(chunk, None);
}

#[test]
fn read_minimal_attempt_read_past_slice_end() {
    let mut file = File::open("test_assets/minimal.riff").unwrap();
    let mut test_data = Vec::new();
    file.read_to_end(&mut test_data).unwrap();
    let mut riff_reader = RiffReader::new(test_data.as_ref());
    let _chunk1 = riff_reader.read_next_chunk().unwrap();
    let chunk2 = riff_reader.read_next_chunk();
    assert_eq!(chunk2, Err(RiffError::EndOfData));
}

#[test]
fn reject_non_ascii_id_constructor() {
    let invalid_id = ChunkId::from_ascii([0xDE, 0xAD, 0xBE, 0xEF]);
    assert_eq!(invalid_id, Err(RiffError::InvalidIDNotASCII));
}

// TODO: More tests
