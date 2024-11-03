use std::mem::size_of_val;

use binrw::prelude::*;
use binrw::BinRead;

#[binrw::parser(reader, endian)]
fn recursive_chunk(chunk_size: u32) -> BinResult<Vec<Chunk>> {
    let mut remaining_bytes = chunk_size as usize;

    let mut result = Vec::new();
    while remaining_bytes > 0 {
        let chunk = Chunk::read_options(reader, endian, ())?;
        let size_of = size_of_val(&chunk);
        if remaining_bytes < size_of {
            break;
        }
        remaining_bytes -= size_of;
        result.push(chunk);
    }

    Ok(result)
}

#[binread]
#[derive(Debug)]
pub enum Chunk {
    #[br(magic = b"FORM")]
    Form {
        chunk_size: u32,
        #[br(parse_with = recursive_chunk, args(chunk_size))]
        children: Vec<Chunk>,
    },
    Record {
        header: u32,
        chunk_size: u32,
        #[br(count = chunk_size)]
        data: Vec<u8>,
    },
}

#[binread]
#[derive(Debug)]
#[br(magic = b"FORM")]
pub struct IFFFile {
    #[br(temp)]
    chunk_size: u32,
    #[br(count = chunk_size)]
    pub data: Vec<u8>,
}
