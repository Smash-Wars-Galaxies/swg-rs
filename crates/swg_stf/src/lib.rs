//! # STF Format Documentation
//!
//! This crate provides utilities to read and extract data from the **STF** format used by
//! the game *Star Wars Galaxies*. The STF format is a custom binary format that stores a list of string keys and values
//! within a single file. STF files are typically identified with the `.stf` extension.
//!
//! ## File Structure
//!
//! A STF file consists of a magic number, followed by a list of Value entries, and a list of Key entries.
//!
//! | Offset (bytes) | Field                  | Description                                                |
//! |----------------|------------------------|------------------------------------------------------------|
//! | 0x0000         | Magic number           | 8 bytes: 0x0000ABCD                                        |
//! | 0x0008         | Unknown Flag           | 1 bytes: A flag with a currently unknown purpose           |
//! | 0x0009         | Max Index              | 4 bytes: The next index to use for each key/value pair     |
//! | 0x000D         | Entry Count            | 4 bytes: The number of entries in this file                |
//!
//! ### Header
//!
//! The STF header consists of the following fields:
//!
//! - **Magic Number**: A 8-byte identifier set to `0x0000ABCD`. This helps identify the file type.
//! - **Unknown Flag**: A 1-byte flag with a currently unknown purpose.
//! - **Max Index**: A 4-byte unsigned integer indicating the current max integer index of the data, starting from 1.
//! - **Entry Count**: A 4-byte unsigned integer indicating the number of records in the archive.
//!
//! ### Value List
//!
//! After the header, the STF file contains a list of records for the final mapping values. These records are stored
//! sequentially. Each record has the following structure:
//!
//! | Offset (bytes) | Field                  | Description                                             |
//! |----------------|------------------------|---------------------------------------------------------|
//! | 0x0000         | ID                     | 4 bytes: Index for current entry                        |
//! | 0x0004         | Unknown                | 4 bytes: Seems to be fixed value of 0xFFFFFFFF          |
//! | 0x0008         | Characters             | 4 bytes: Number of characters in the string             |
//! | 0x000B         | Data                   | (Characters * 2) bytes: UTF16 string                    |
//!
//! ### Key List
//!
//! After the value list, the STF file contains a list of records for the final mapping keys. These records are stored
//! sequentially. Each record has the following structure:
//!
//! | Offset (bytes) | Field                  | Description                                             |
//! |----------------|------------------------|---------------------------------------------------------|
//! | 0x0000         | ID                     | 4 bytes: Index for current entry                        |
//! | 0x0004         | Characters             | 4 bytes: Number of characters in the string             |
//! | 0x0008         | Data                   | (Characters) bytes: UTF8 string                         |
//!
//! ## Additional Information
//!
//! - **File Extension**: `.stf`
//! - **Endianness**: Little-endian for all multi-byte integers
//!

pub mod error;
pub mod read;

pub use read::StringTableReader;
