//! This library handles reading from and creating **TRE** files used by *Star Wars Galaxies*.
//! 
//! # TRE Archive Format Documentation
//!
//! This crate provides utilities to read and extract data from the **TRE** archive format used by
//! the game *Star Wars Galaxies*. The TRE format is a custom binary format that stores various game assets
//! within a single file. TRE files are typically identified with the `.tre` extension.
//!
//! ## File Structure
//!
//! A TRE file consists of a header, followed by the data blocks, a metadata block for records, and a name block.
//!
//! | Offset (bytes) | Field                  | Description                                                |
//! |----------------|------------------------|------------------------------------------------------------|
//! | 0x0000         | Magic number           | 4 bytes: 0x54524545 ("TREE")                               |
//! | 0x0004         | Version                | 4 bytes: Fixed value 0x00000005 ("0005")                   |
//! | 0x0008         | Record Count           | 4 bytes: Number of records in the archive                  |
//! | 0x000C         | Record Offset          | 4 bytes: Offset to the record metadata block               |
//! | 0x0010         | Record Compression     | 4 bytes: Compression method for records                    |
//! | 0x0014         | Record Comp. Size      | 4 bytes: Compressed size of record block                   |
//! | 0x0018         | Name Compression       | 4 bytes: Compression method for names                      |
//! | 0x001C         | Name Comp. Size        | 4 bytes: Compressed size of the name block                 |
//! | 0x0020         | Name Uncomp. Size      | 4 bytes: Uncompressed size of the name block               |
//!
//! ### Header
//!
//! The TRE header consists of the following fields:
//!
//! - **Magic Number**: A 4-byte identifier set to `0x54524545`, which is the ASCII code for "TREE". This helps
//!   identify the file type.
//! - **Version**: A 4-byte unsigned integer representing the version of the TRE format. The version is fixed
//!   at `0x00000005` ("0005").
//! - **Record Count**: A 4-byte unsigned integer indicating the number of records in the archive.
//! - **Record Block Offset**: A 4-byte unsigned integer specifying the offset to the start of the record
//!   metadata block from the beginning of the file.
//! - **Record Block Compression**: A 4-byte unsigned integer indicating the compression method used for the
//!   entire record metadata block. Possible values are:
//!   - `0`: None (no compression)
//!   - `2`: Zlib (compressed with Zlib)
//! - **Record Block Compressed Size**: A 4-byte unsigned integer representing the compressed size of the record
//!   metadata block, if compression is applied.
//! - **Name Block Compression**: A 4-byte unsigned integer indicating the compression method for the entire
//!   name block. Possible values are:
//!   - `0`: None (no compression)
//!   - `2`: Zlib (compressed with Zlib)
//! - **Name Block Compressed Size**: A 4-byte unsigned integer for the compressed size of the name block,
//!   if compression is applied.
//! - **Name Block Uncompressed Size**: A 4-byte unsigned integer specifying the uncompressed size of the name block.
//!
//! ### Data Blocks
//!
//! After the header, the TRE file contains the actual data blocks for each record. These blocks are stored
//! sequentially and may be compressed depending on the specified compression method. Each record's data is stored
//! either in its uncompressed or compressed form.
//!
//! ### Record Metadata Block
//!
//! The record metadata block contains entries that describe each file stored in the TRE archive. The entire
//! block may be compressed depending on the **Record Block Compression** method specified in the header. If
//! compressed, the compressed size is given by **Record Block Compressed Size**. Each record has the following
//! structure:
//!
//! | Offset (bytes) | Field                  | Description                                             |
//! |----------------|------------------------|---------------------------------------------------------|
//! | 0x0000         | CRC32                  | 4 bytes: CRC-32 checksum of the record data             |
//! | 0x0004         | Uncompressed Size      | 4 bytes: Size of the data when uncompressed             |
//! | 0x0008         | Data Offset            | 4 bytes: Offset to the start of the record data block   |
//! | 0x000C         | Compression            | 4 bytes: Compression method for the record data         |
//! | 0x0010         | Compressed Size        | 4 bytes: Compressed size of the record data             |
//! | 0x0014         | Name Offset            | 4 bytes: Offset to the name within the name block       |
//!
//! - **CRC32**: A 4-byte checksum used to verify the integrity of the record data.
//! - **Uncompressed Size**: A 4-byte integer indicating the size of the data when fully uncompressed.
//! - **Data Offset**: A 4-byte integer specifying the offset from the start of the file to the record data block.
//! - **Compression**: A 4-byte unsigned integer representing the compression method for this specific record.
//!   Possible values are:
//!   - `0`: None (no compression)
//!   - `2`: Zlib (compressed with Zlib)
//! - **Compressed Size**: A 4-byte integer specifying the size of the data in compressed form.
//! - **Name Offset**: A 4-byte integer indicating the offset to the name of this record within the name block.
//!
//! ### Name Block
//!
//! The name block stores the file paths or names associated with each record. The entire name block may be
//! compressed depending on the **Name Block Compression** method specified in the header. If compressed, the
//! compressed size is given by **Name Block Compressed Size**, while the uncompressed size is specified by
//! **Name Block Uncompressed Size**. The names are stored sequentially as UTF-8 strings, each ending with a
//! null terminator. The offsets within the record metadata point to positions within this block.
//!
//! ## Additional Information
//!
//! - **File Extension**: `.tre`
//! - **Endianness**: Little-endian for all multi-byte integers
//! - **Compression Methods**:
//!   - `0`: None (no compression)
//!   - `2`: Zlib (compressed with Zlib)
//!

pub mod compression;
pub mod error;
pub mod read;
pub mod types;
pub mod write;

pub use compression::CompressionMethod;
pub use read::TreArchive;
pub use write::TreWriter;
