// // typedef struct {
// // 	ID	ckID;
// // 	LONG	ckSize;	/* sizeof(ckData) */
// // 	UBYTE	ckData[/* ckSize */];
// // 	} Chunk;

// // #define ID_FORM   MakeID('F','O','R','M')
// // #define ID_LIST   MakeID('L','I','S','T')
// // #define ID_PROP   MakeID('P','R','O','P')
// // #define ID_CAT    MakeID('C','A','T',' ')
// // #define ID_FILLER MakeID(' ',' ',' ',' ')

// use std::io::{self, Read, Seek};

// use crate::error::Result;

pub struct Chunk<R: Read> {
    pub id: [u8; 4],
    pub size: u32,
    pub buffer: io::Take<R>,
}

// impl<R: Read> Chunk<R> {
//     pub fn read(mut reader: R) -> Result<Self> {
//         let mut buffer = [0u8; 4];
//         reader.read_exact(&mut buffer)?;

//         let size = 0u32;
//         reader.read_exact(&mut size.to_le_bytes())?;

//         Ok(Chunk{
//             id: buffer,
//             size,
//             buffer: reader.take(size as u64)
//         })
//     }
// }

// #[cfg(test)]
// mod test {
//     use std::io::Cursor;

//     use tracing_test::traced_test;

//     use crate::types::Chunk;
//     use crate::error::Result;

//     #[traced_test]
//     #[test]
//     fn read_chunk_minimum() -> Result<()> {
//         #[rustfmt::skip]
//         let mut input = Cursor::new(vec![
//             b'F', b'O', b'R', b'M',  // ID
//             0x00, 0x00, 0x00, 0x00, // Size
//         ]);

//         let chunk = Chunk::read(&mut input)?;

//         assert_eq!(chunk.id, *b"FORM");
//         assert_eq!(chunk.size, 0);

//         Ok(())
//     }

//     #[traced_test]
//     #[test]
//     fn write_chunk_minimum() {
//         #[rustfmt::skip]
//         let mut input = Cursor::new(vec![
//             b'F', b'O', b'R', b'M',  // ID
//             0x00, 0x00, 0x00, 0x00, // Size
//         ]);
//     }

//     #[traced_test]
//     #[test]
//     fn read_chunk_with_sub_chunk() -> Result<()> {
//         #[rustfmt::skip]
//         let mut input = Cursor::new(vec![
//             b'F', b'O', b'R', b'M',  // ID
//             0x0B, 0x00, 0x00, 0x00, // Size
//             b'D', b'T', b'I', b'I',  // ID
//             b'F', b'O', b'R', b'M',  // ID
//             0x00, 0x00, 0x00, 0x00, // Size
//         ]);

//         let chunk = Chunk::read(&mut input)?;

//         assert_eq!(chunk.id, *b"FORM");
//         assert_eq!(chunk.size, 12);

//         Ok(())
//     }

//     /* Odd numbers of bytes should be padded */
//     #[traced_test]
//     #[test]
//     fn read_chunk_with_padding() {
//         #[rustfmt::skip]
//         let mut input = Cursor::new(vec![
//             b'F', b'O', b'R', b'M',  // ID
//             0x00, 0x00, 0x00, 0x00, // Size
//         ]);
//     }

//     #[traced_test]
//     #[test]
//     fn write_chunk_with_padding() {
//         #[rustfmt::skip]
//         let mut input = Cursor::new(vec![
//             b'F', b'O', b'R', b'M',  // ID
//             0x00, 0x00, 0x00, 0x00, // Size
//         ]);
//     }
// }
