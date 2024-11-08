use core::str;
use std::path::Path;

use crate::error::Error;
use crate::error::Result;
use miette::miette;
use winnow::ascii::digit1;
use winnow::binary::le_u32;
use winnow::binary::u8;
use winnow::combinator::empty;
use winnow::combinator::fail;
use winnow::combinator::peek;
use winnow::combinator::rest;
use winnow::combinator::{seq, dispatch, trace};
use winnow::error::InputError;
use winnow::prelude::*;
use winnow::token::literal;
use winnow::token::one_of;
use winnow::token::take;
use winnow::PResult;
use winnow::Parser;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Chunk<'a> {
    pub id: &'a str,
    pub size: u32,
    pub data: &'a [u8],
}

fn parse_chunk<'s>(s: &mut &'s [u8]) -> PResult<Chunk<'s>> { 
    seq!( Chunk {
        id: take(4u8).try_map(|c| str::from_utf8(c)),
        size: le_u32,
        data: take(size),
    })
    .parse_next(s)

}

impl Chunk<'_> {
    pub fn parse(data: &[u8]) -> Result<Chunk<'_>> {
        let mut buf = data;
        Ok(parse_chunk(&mut buf).unwrap())
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

// pub(crate) type Stream<'i> = &'i [u8];

// pub fn parse(data: &[u8]) -> PResult<Chunk<'_>, InputError<&'_ [u8]>> {
//     let mut buf = data;
//     parse_form_chunk(&mut buf)
// }

// fn parse_chunk<'s>(input: &mut Stream<'s>) -> PResult<Chunk<'s>> {
//     dispatch! {
//         peek::<_, &[u8],_,_>(take(4u8));
//         b"FORM" => parse_chunk_form,
//         _ => fail::<_, &str,_>,
//     }
//     .parse_next(input)
// }

// fn parse_chunk_form<'s>(input: &mut Stream<'s>) -> PResult<Chunk<'s>> {
// }

// // fn request_line<'s>(input: &mut Stream<'s>) -> PResult<Request<'s>> {
// //     seq!( Request {
// //         method: take_while(1.., is_token),
// //         _: take_while(1.., is_space),
// //         uri: take_while(1.., is_not_space),
// //         _: take_while(1.., is_space),
// //         version: http_version,
// //         _: line_ending,
// //     })
// //     .parse_next(input)
// // }

// pub fn parse_id<'s>(input: &mut Stream<'s>) -> PResult<&'s str, InputError<&'s [u8]>> {
//     literal("FORM")
//         .try_map(|d| str::from_utf8(d))
//         .parse_next(input)
// }

// pub fn parse_size<'s>(s: &mut &'s [u8]) -> PResult<u32, InputError<&'s [u8]>> {
//     trace("parse_size", |i: &mut _| le_u32.parse_next(i)).parse_next(s)
// }

// pub fn parse_form_chunk<'s>(s: &mut &'s [u8]) -> PResult<Chunk<'s>, InputError<&'s [u8]>> {
//     let (id, size) = (parse_id, parse_size).parse_next(s)?;

//     Ok(Chunk {
//         id,
//         size,
//         children: Vec::new(),
//     })
// }


#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Form<'a> {
    pub size: u32,
    pub id: &'a str,
    pub children: Vec<Chunk<'a>>
}

/*
* Group chunk IDs
	FORM, LIST, PROP, CAT.
Future revision group chunk IDs
	FOR1 I FOR9, LIS1 I LIS9, CAT1 I CAT9.
*/
pub enum GroupChunk<'a> {
    // FORM	::= "FORM" #{ FormType (LocalChunk | FORM | LIST | CAT)* }
    // FormType	::= ID
    // LocalChunk	::= Property | Chunk
    Form{
        id: &'a str
    },
    // LIST	::= "LIST" #{ ContentsType PROP* (FORM | LIST | CAT)* }
    // ContentsType	::= ID
    List{
        id: &'a str
    },
    // PROP	::= "PROP" #{ FormType Property* }
    Property{
        id: &'a str
    },
    // CAT	::= "CAT " #{ ContentsType (FORM | LIST | CAT)* }
    // ContentsType	::= ID	-- a hint or an "abstract data type" ID
    Concatonation{
        id: &'a str
    },
}

/* 
FORM type IDs
	(The above group chunk IDs may not be used for FORM type IDs.)
	(Lower case letters and punctuation marks are forbidden in FORM 
type IDs.)
*/

fn group_chunk_id<'s>(s: &mut &'s [u8]) -> PResult<&'s str> {
    dispatch! {
        peek::<_, &[u8],_,_>(take(4u8));
        b"FORM" => take(4u8).try_map(|c| str::from_utf8(c)),
        b"LIST" => take(4u8).try_map(|c| str::from_utf8(c)),
        b"PROP" => take(4u8).try_map(|c| str::from_utf8(c)),
        b"CAT " => take(4u8).try_map(|c| str::from_utf8(c)),
        _ => fail::<_, &str,_>,
    }
    .parse_next(s)
}

fn parse_form<'s>(s: &mut &'s [u8]) -> PResult<Option<Form<'s>>> {
    let chunk = parse_chunk(s)?;
    if chunk.id != "FORM" {
        return Ok(None);
    }

    let mut buf = chunk.data;
    let id = take(4u8).try_map(|c| str::from_utf8(c)).parse_next(&mut buf)?;

    Ok(Some(Form{size: chunk.size, id, children: Vec::new() }))
}

impl Form<'_> {
    pub fn parse(data: &[u8]) -> Result<Option<Form<'_>>> {
        let mut buf = data;
        Ok(parse_form(&mut buf).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;
    use winnow::prelude::*;
    use winnow::{Bytes, Parser};

    use crate::iff::{Form, Chunk};

    #[traced_test]
    #[test]
    fn read_chunk_minimum() {
        #[rustfmt::skip]
        let input = vec![
            b'F', b'O', b'R', b'M',  // ID
            0x04, 0x00, 0x00, 0x00,  // Size
            b'D', b'T', b'I', b'I',  // ID
        ];

        let result = Chunk::parse(input.as_slice()).unwrap();

        assert_eq!(
            result,
            Chunk {
                id: "FORM",
                size: 4,
                data: b"DTII"
            }
        );
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

        let result = Chunk::parse(input.as_slice()).unwrap();

        assert_eq!(
            result,
            Chunk {
                id: "FORM",
                size: 4,
                data: b"DTII"
            }
        );
    }

    #[traced_test]
    #[test]
    fn read_form() {
        #[rustfmt::skip]
        let input = vec![
            b'F', b'O', b'R', b'M',  // ID
            0x04, 0x00, 0x00, 0x00,  // Size
            b'D', b'T', b'I', b'I',  // ID
        ];

        let result = Form::parse(input.as_slice()).unwrap();

        assert_eq!(
            result,
            Some(Form {
                id: "DTII",
                size: 4,
                children: Vec::new()
            })
        );
    }

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

        let result = Form::parse(input.as_slice()).unwrap();

        assert_eq!(
            result,
            Some(Form {
                id: "DTII",
                size: 4,
                children: vec![
                    Chunk {
                        id: "CRAC",
                        size: 13,
                        data: &[0x68, 0x65, 0x6C, 0x6C, 0x6F, 0x2C, 0x77, 0x6F, 0x72, 0x6C, 0x64, 0x21, 0x0A]
                    }
                ]
            })
        );
    }


    
}
