//! Types for reading string table files
//!

use byteorder::{LittleEndian, ReadBytesExt};
use std::{
    collections::HashMap,
    io::{Read, Seek},
};
use widestring::U16String;

use crate::{
    error::{Error, Result},
    types::StringTable,
};

/// STF file reader
///
/// ```no_run
/// use std::io::prelude::*;
///
/// fn list_entries(reader: impl Read + Seek) -> swg_stf::error::Result<()> {
///     let stf = swg_stf::TreArchive::new(reader)?;
///
///     for (key, value) in stf.get_entries() {
///         println!("{}: {}", &key, &value.display());
///     }
///
///     Ok(())
/// }
/// ```
pub struct StringTableReader {}

impl StringTableReader {
    /// Read a STF file and parse it's entries.
    pub fn decode<R: Read + Seek>(mut reader: R) -> Result<StringTable> {
        let magic = reader.read_u32::<LittleEndian>()?;
        if magic != 0x0000ABCD {
            return Err(Error::InvalidFile);
        }

        let _flag = reader.read_u8()?;
        let _next_index = reader.read_u32::<LittleEndian>()?;
        let count = reader.read_u32::<LittleEndian>()?;

        let mut values = HashMap::with_capacity(count as usize);
        for _ in 0..count {
            let id = reader.read_u32::<LittleEndian>()?;
            let _unknown = reader.read_u32::<LittleEndian>()?; // 0xFFFFFFFF
            let runes = reader.read_u32::<LittleEndian>()? as usize;

            let mut buffer = Vec::with_capacity(runes);
            for _ in 0..runes {
                let rune = reader.read_u16::<LittleEndian>()?;
                buffer.push(rune);
            }

            values.insert(id, U16String::from_vec(buffer));
        }

        let mut names = HashMap::with_capacity(count as usize);
        for _ in 0..count {
            let id = reader.read_u32::<LittleEndian>()?;
            let runes = reader.read_u32::<LittleEndian>()? as usize;

            let mut buffer = Vec::with_capacity(runes);
            for _ in 0..runes {
                let rune = reader.read_u8()?;
                buffer.push(rune);
            }

            names.insert(id, String::from_utf8(buffer)?);
        }

        let entries = names
            .iter()
            .filter_map(|(id, name)| values.get(id).map(|value| (name.clone(), value.clone())))
            .collect();

        Ok(StringTable::new(entries))
    }
}
