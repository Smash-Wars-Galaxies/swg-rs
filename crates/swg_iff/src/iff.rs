use core::str;
use std::any::Any;
use std::fmt;
use std::path::Path;

use crate::error::Result;
use winnow::binary::be_u32;
use winnow::combinator::peek;
use winnow::combinator::repeat;
use winnow::combinator::{dispatch, seq, trace};
use winnow::prelude::*;
use winnow::token::take;
use winnow::Bytes;
use winnow::PResult;
use winnow::Parser;
use winnow::Partial;

type Stream<'i> = Partial<&'i Bytes>;

fn stream(b: &[u8]) -> Stream<'_> {
    Partial::new(Bytes::new(b))
}

fn complete_stream(b: &[u8]) -> Stream<'_> {
    let mut p = Partial::new(Bytes::new(b));
    let _ = p.complete();
    p
}

pub trait Chunk<'a> {
    type Item;
    fn parse(data: &'a [u8]) -> Result<Self::Item>
    where
        Self: Sized;

    fn id(&self) -> &'_ str;
    fn children(&self) -> Vec<Box<dyn Any>>;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum GroupChunk<'a> {
    Chunk {
        id: &'a str,
        size: u32,
        data: &'a [u8],
    },
    // FORM	::= "FORM" #{ FormType (LocalChunk | FORM | LIST | CAT)* }
    // FormType	::= ID
    // LocalChunk	::= Property | Chunk
    Form {
        form_type: &'a str,
        children: Vec<GroupChunk<'a>>,
    },
    // LIST	::= "LIST" #{ ContentsType PROP* (FORM | LIST | CAT)* }
    // ContentsType	::= ID
    List {
        content_type: &'a str,
        children: Vec<GroupChunk<'a>>,
    },
    // PROP	::= "PROP" #{ FormType Property* }
    Property {
        content_type: &'a str,
    },
    // CAT	::= "CAT " #{ ContentsType (FORM | LIST | CAT)* }
    // ContentsType	::= ID	-- a hint or an "abstract data type" ID
    Concatonation {
        content_type: &'a str,
        children: Vec<GroupChunk<'a>>,
    },
}

fn parse_generic_chunk<'s>(s: &mut Stream<'s>) -> PResult<GroupChunk<'s>> {
    seq!(GroupChunk::Chunk {
        id: take(4u8).try_map(|c| str::from_utf8(c)),
        size: be_u32,
        data: take(size),
    })
    .parse_next(s)
}

fn parse_form_chunk<'s>(s: &mut Stream<'s>) -> PResult<GroupChunk<'s>> {
    seq!(GroupChunk::Form {
        _: take(4u8).try_map(|c| str::from_utf8(c)),
        _: be_u32,
        form_type: take(4u8).try_map(|c| str::from_utf8(c)),
        children: repeat(0.., parse_group_chunk),
    })
    .parse_next(s)
}

fn parse_group_chunk<'s>(s: &mut Stream<'s>) -> PResult<GroupChunk<'s>> {
    dispatch! {
        peek::<_, &[u8],_,_>(take(4u8));
        b"FORM" => parse_form_chunk,
        // b"LIST" => take(4u8).try_map(|c| str::from_utf8(c)),
        // b"PROP" => take(4u8).try_map(|c| str::from_utf8(c)),
        // b"CAT " => take(4u8).try_map(|c| str::from_utf8(c)),
        _ => parse_generic_chunk,
    }
    .parse_next(s)
}

impl<'a> GroupChunk<'a> {
    pub fn parse(data: &'a [u8]) -> Result<GroupChunk<'a>> {
        let mut buf = complete_stream(data);
        Ok(parse_group_chunk(&mut buf).unwrap())
    }
}

pub struct IFFReader {}

impl IFFReader {
    pub fn parse(path: impl AsRef<Path>) -> Result<Vec<u8>> {
        let data = std::fs::read(path)?;
        Ok(data)

        // let output = parse_form_chunk.parse_next(&mut data.as_slice()).unwrap();

        // Ok(output.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use tracing_test::traced_test;

    use crate::iff::GroupChunk;

    #[traced_test]
    #[test]
    fn read_chunk_minimum() {
        #[rustfmt::skip]
        let input = vec![
            b'F', b'O', b'R', b'M',  // ID
            0x00, 0x00, 0x00, 0x00,  // Size
        ];

        // let result = Form::parse(input.as_slice()).unwrap();

        // assert_eq!(
        //     result,
        //     Form {
        //         size: 0,
        //         data: &[],
        //         tag: ""
        //     }
        // );
    }

    #[traced_test]
    #[test]
    fn read_chunk_with_data() {
        #[rustfmt::skip]
        let input = vec![
            b'F', b'O', b'R', b'M',  // ID
            0x04, 0x00, 0x00, 0x00,  // Size
            b'D', b'T', b'I', b'I',  // ID
        ];

        // let result = Form::parse(input.as_slice()).unwrap();

        // assert_eq!(
        //     result,
        //     Form {
        //         size: 4,
        //         data: b"DTII",
        //         tag: "DTII"
        //     }
        // );
    }

    // #[traced_test]
    // #[test]
    // fn read_form() {
    //     #[rustfmt::skip]
    //     let input = vec![
    //         b'F', b'O', b'R', b'M',  // ID
    //         0x04, 0x00, 0x00, 0x00,  // Size
    //         b'D', b'T', b'I', b'I',  // ID
    //     ];

    //     let result = Form::parse(input.as_slice()).unwrap();

    //     assert_eq!(
    //         result,
    //         Some(Form {
    //             id: "DTII",
    //             size: 4,
    //             children: Vec::new()
    //         })
    //     );
    // }

    #[traced_test]
    #[test]
    fn read_example() {
        #[rustfmt::skip]
        let input = vec![
            b'F', b'O', b'R', b'M',  // ID
            0x00, 0x00, 0x00, 0x1A,  // Size
            b'S', b'N', b'A', b'P',  // Type

            b'C', b'R', b'A', b'C',  // Chunk ID
            0x00, 0x00, 0x00, 0x0D,  // Size
            0x68, 0x65, 0x6C, 0x6C, 0x6F, 0x2C, 0x77, 0x6F, 0x72, 0x6C, 0x64, 0x21, 0x0A, 0x00,
        ];

        let result = GroupChunk::parse(input.as_slice()).unwrap();

        assert_eq!(
            result,
            GroupChunk::Form {
                form_type: "SNAP",
                children: vec![GroupChunk::Chunk {
                    id: "CRAC",
                    size: 13,
                    data: &[
                        0x68, 0x65, 0x6C, 0x6C, 0x6F, 0x2C, 0x77, 0x6F, 0x72, 0x6C, 0x64, 0x21,
                        0x0A
                    ]
                }]
            }
        );
    }

    #[traced_test]
    #[test]
    fn test_interleaved_bitmap_example() {
        #[rustfmt::skip]
        let ilbm_data = [
            // FORM chunk header
            0x46, 0x4F, 0x52, 0x4D, // "FORM"
            0x00, 0x00, 0x00, 0x34, // Chunk size (52 bytes including header)
            0x49, 0x4C, 0x42, 0x4D, // "ILBM" (ILBM image identifier)

            // BMHD (Bitmap Header) chunk
            0x42, 0x4D, 0x48, 0x44, // "BMHD"
            0x00, 0x00, 0x00, 0x14, // Chunk size (20 bytes)
            0x00, 0x40, 0x00, 0x40, // Width (64) and Height (64)
            0x00, 0x00, 0x00, 0x00, // X and Y position (0,0)
            0x00, 0x08,             // Bitplane depth (8-bit color)
            0x00,                   // Masking (none)
            0x00,                   // Compression (none)
            0x00, 0x00,             // Transparent color index (0)
            0x00, 0x00,             // X and Y aspect ratio (1:1)
            0x00, 0x40, 0x00, 0x40, // Page width and height (64x64)

            // CMAP (Color Map) chunk
            0x43, 0x4D, 0x41, 0x50, // "CMAP"
            0x00, 0x00, 0x00, 0x0C, // Chunk size (12 bytes for 4 colors)
            0x00, 0x00, 0x00,       // Color 1 (black)
            0xFF, 0xFF, 0xFF,       // Color 2 (white)
            0xFF, 0x00, 0x00,       // Color 3 (red)
            0x00, 0xFF, 0x00,       // Color 4 (green)

            // BODY (Image Data) chunk
            0x42, 0x4F, 0x44, 0x59, // "BODY"
            0x00, 0x00, 0x00, 0x08, // Chunk size (8 bytes)
            0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x11 // Image data bytes
        ];

        let expected = GroupChunk::Form {
            form_type: "ILBM",
            children: vec![
                GroupChunk::Chunk {
                    id: "BMHD",
                    size: 20,
                    data: &[
                        0x00, 0x40, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x40,
                    ],
                },
                GroupChunk::Chunk {
                    id: "CMAP",
                    size: 12,
                    data: &[
                        0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0xFF, 0x00,
                    ],
                },
                GroupChunk::Chunk {
                    id: "BODY",
                    size: 8,
                    data: &[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x11],
                },
            ],
        };

        let result = GroupChunk::parse(&ilbm_data).unwrap();
        assert_eq!(result, expected);
    }
}
