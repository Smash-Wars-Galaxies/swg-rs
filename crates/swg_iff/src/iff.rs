use core::str;
use std::collections::HashMap;
use std::fmt;
use std::path::Path;

use crate::error::Result;
use better_any::tid;
use better_any::Tid;
use better_any::TidAble;
use better_any::TidExt;
use downcast_rs::impl_downcast;
use downcast_rs::Downcast;
use miette::miette;
use tracing::info;
use winnow::binary::be_u32;
use winnow::combinator::empty;
use winnow::combinator::fail;
use winnow::combinator::peek;
use winnow::combinator::repeat;
use winnow::combinator::{dispatch, seq, trace};
use winnow::error::ContextError;
use winnow::prelude::*;
use winnow::token::take;
use winnow::Bytes;
use winnow::PResult;
use winnow::Parser;
use winnow::Partial;
use std::fmt::Debug;

type Stream<'i> = &'i [u8];

fn stream<'a>(b: &'a [u8]) -> Stream<'a> {
    Bytes::new(b)
}

// fn parse_generic_chunk<'s>(s: &mut Stream<'s>) -> PResult<GroupChunk<'s>> {
//     seq!(GroupChunk::Chunk {
//         id: take(4u8).try_map(|c| str::from_utf8(c)),
//         size: be_u32,
//         data: take(size),
//     })
//     .parse_next(s)
// }

pub trait ChunkBuilder {
    fn build<'s>(&'s self, s: &mut Stream<'s>) -> PResult<Box<dyn Tid<'s> + 's>>;
}
tid!(ChunkBuilder);

#[derive(Default, Tid)]
pub struct FormBuilder;

impl ChunkBuilder for FormBuilder {
    fn build<'s>(&self, s: &mut Stream<'s>) -> PResult<Box<dyn Tid<'s> + 's>> {
        let form = Form::parse(s)?;

        Ok(Box::new(form))
    }
}

pub trait Chunk<'a> {
    fn id(&self) -> &'a str;
    // fn children<'a>(&self) -> &Vec<Box<dyn Chunk + 'a>>;
}
tid!{ impl<'b> TidAble<'b> for dyn Chunk<'b> + 'b }

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Tid)]
pub struct Form<'a> {
    form_type: &'a str,
    // children: Vec<Box<dyn Chunk + 'a>>,
}

// impl PartialEq for Form<'_> {
//     fn eq(&self, other: &Self) -> bool {
//         self.form_type == other.form_type
//             /*&& self
//                 .children
//                 .iter()
//                 .zip(other.children.iter())
//                 .all(|(a, b)| (*(*a)).type_id() == (*(*b)).type_id())*/
//     }
// }

impl Form<'_> {
    fn parse<'s>(s: &mut Stream<'s>) -> PResult<Form<'s>> {
        be_u32.parse_next(s)?;
        let form_type = take(4u8).try_map(|c| str::from_utf8(c)).parse_next(s)?;
        Ok(Form {
            form_type,
            ..Default::default()
        })
    }
}

impl<'a> Chunk<'a> for Form<'a> {
    // type Item = Form<'a>;

    fn id(&self) -> &'a str {
        self.form_type
    }

    // fn children<'a>(&self) -> &Vec<Box<dyn Chunk + 'a>> {
    //     &self.children
    // }
}

// #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
// pub enum GroupChunk<'a> {
//     Chunk {
//         id: &'a str,
//         size: u32,
//         data: &'a [u8],
//     },
//     // FORM	::= "FORM" #{ FormType (LocalChunk | FORM | LIST | CAT)* }
//     // FormType	::= ID
//     // LocalChunk	::= Property | Chunk
//     Form {
//         form_type: &'a str,
//         children: Vec<GroupChunk<'a>>,
//     },
//     // LIST	::= "LIST" #{ ContentsType PROP* (FORM | LIST | CAT)* }
//     // ContentsType	::= ID
//     List {
//         content_type: &'a str,
//         children: Vec<GroupChunk<'a>>,
//     },
//     // PROP	::= "PROP" #{ FormType Property* }
//     Property {
//         content_type: &'a str,
//     },
//     // CAT	::= "CAT " #{ ContentsType (FORM | LIST | CAT)* }
//     // ContentsType	::= ID	-- a hint or an "abstract data type" ID
//     Concatonation {
//         content_type: &'a str,
//         children: Vec<GroupChunk<'a>>,
//     },
// }

// fn parse_form_chunk<'s>(s: &mut Stream<'s>) -> PResult<GroupChunk<'s>> {
//     seq!(GroupChunk::Form {
//         _: take(4u8).try_map(|c| str::from_utf8(c)),
//         _: be_u32,
//         form_type: take(4u8).try_map(|c| str::from_utf8(c)),
//         children: repeat(0.., parse_group_chunk),
//     })
//     .parse_next(s)
// }

// fn parse_group_chunk<'s>(s: &mut Stream<'s>) -> PResult<GroupChunk<'s>> {
//     dispatch! {
//         peek::<_, &[u8],_,_>(take(4u8));
//         b"FORM" => parse_form_chunk,
//         // b"LIST" => take(4u8).try_map(|c| str::from_utf8(c)),
//         // b"PROP" => take(4u8).try_map(|c| str::from_utf8(c)),
//         // b"CAT " => take(4u8).try_map(|c| str::from_utf8(c)),
//         _ => parse_generic_chunk,
//     }
//     .parse_next(s)
// }

// impl<'a> GroupChunk<'a> {
//     pub fn parse(data: &'a [u8]) -> Result<GroupChunk<'a>> {
//         let mut buf = stream(data);
//         Ok(parse_group_chunk(&mut buf).unwrap())
//     }
// }

#[derive(Default)]
pub struct IFFReader {
    registry: HashMap<&'static str, Box<dyn ChunkBuilder>>,
}

impl IFFReader {
    pub fn new() -> Self {
        let mut reader = IFFReader {
            registry: HashMap::new(),
        };
        reader.register_chunk::<FormBuilder>("FORM");
        // Box::new(|d| {
        //     Ok(Box::new(Form::parse(d)?) as Box<dyn Any>)
        // }));
        reader
    }

    pub fn register_chunk<T: ChunkBuilder + Default + 'static>(&mut self, name: &'static str) {
        self.registry.insert(name, Box::new(T::default()));
    }

    pub fn parse(&self, data: Stream<'static>) -> PResult<Form> {
        let mut buf = data;

        let id = take(4u8)
            .try_map(|c| str::from_utf8(c))
            .parse_next(&mut buf)?;

        let parser = self.registry.get(id).unwrap();
        let result = parser.as_ref().build(&mut buf)?;
        println!("IsForm: {}", result.is::<Chunk>());
        Ok(result.downcast_move::<Form>().unwrap())
            // parse.downcast_ref::<&FormBuilder>().unwrap().build(&mut buf).map(|r| r.downcast_move::<Form>().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use tracing_test::traced_test;

    use crate::iff::{Form, IFFReader};

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

        // let result = GroupChunk::parse(input.as_slice()).unwrap();

        // assert_eq!(
        //     result,
        //     GroupChunk::Form {
        //         form_type: "SNAP",
        //         children: vec![GroupChunk::Chunk {
        //             id: "CRAC",
        //             size: 13,
        //             data: &[
        //                 0x68, 0x65, 0x6C, 0x6C, 0x6F, 0x2C, 0x77, 0x6F, 0x72, 0x6C, 0x64, 0x21,
        //                 0x0A
        //             ]
        //         }]
        //     }
        // );
    }

    #[traced_test]
    #[test]
    fn test_interleaved_bitmap_example() {
        #[rustfmt::skip]
        let ilbm_data: &[u8] = &[
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

        let parser = IFFReader::new();

        let expected = Form {
            form_type: "ILBM",
            // children: vec![
                // GroupChunk::Chunk {
                //     id: "BMHD",
                //     size: 20,
                //     data: &[
                //         0x00, 0x40, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00,
                //         0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x40,
                //     ],
                // },
                // GroupChunk::Chunk {
                //     id: "CMAP",
                //     size: 12,
                //     data: &[
                //         0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0xFF, 0x00,
                //     ],
                // },
                // GroupChunk::Chunk {
                //     id: "BODY",
                //     size: 8,
                //     data: &[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x11],
                // },
            // ],
        };

        let result = parser.parse(ilbm_data).unwrap();
        assert_eq!(result, expected);
    }
}
