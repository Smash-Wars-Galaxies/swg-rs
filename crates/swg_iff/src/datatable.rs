use core::str;
use std::io::Cursor;
use std::str::FromStr;

use crate::error::Error;

use binrw::prelude::*;
use binrw::BinRead;
use binrw::NullString;

#[derive(Debug)]
pub struct Cell {
    pub name: NullString,
    pub data: CellData,
    pub cell_type: CellType,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CellData {
    String(NullString),
    Boolean(bool),
    Integer(u32),
    Enum(u32),
}

#[binrw::parser(reader, endian)]
fn cell_parser(columns: &Vec<NullString>, types: &Vec<CellType>) -> BinResult<Vec<Cell>> {
    let mut result = Vec::new();
    for (i, column_name) in columns.iter().enumerate() {
        result.push(Cell {
            name: column_name.to_owned(),
            data: match types[i] {
                CellType::String(_) => {
                    CellData::String(NullString::read_options(reader, endian, ())?)
                }
                CellType::Boolean(_) => CellData::Boolean(u32::read_ne(reader)? != 0),
                CellType::Integer(_) => CellData::Integer(u32::read_ne(reader)?),
                CellType::Enum(_, _) => CellData::Enum(u32::read_ne(reader)?),
            },
            cell_type: types[i].clone(),
        })
    }
    Ok(result)
}

#[binread]
#[derive(Debug)]
#[br(import(columns: &Vec<NullString>, types: &Vec<CellType>))]
pub struct Row {
    #[br(parse_with = cell_parser, args(columns, types))]
    pub cells: Vec<Cell>,
}

#[derive(Debug, Clone)]
pub enum CellType {
    String(String),
    Enum(Vec<String>, usize),
    Boolean(bool),
    Integer(u32),
}

impl TryFrom<&NullString> for CellType {
    type Error = Error;

    fn try_from(value: &NullString) -> Result<Self, Self::Error> {
        str::from_utf8(&value.0)
            .map_err(Error::Utf8Error)
            .and_then(Self::from_str)
    }
}

impl FromStr for CellType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let default = if s.contains('[') && s.contains(']') {
            Some(&s[s.find('[').unwrap() + 1..s.find(']').unwrap()])
        } else {
            None
        };

        let options = if s.contains('(') && s.contains(')') {
            let section = s[s.find('(').unwrap() + 1..s.find(')').unwrap()]
                .split(',')
                .filter_map(|c| c.split('=').next().map(|s| s.to_string()))
                .collect::<Vec<String>>();
            Some(section)
        } else {
            None
        };

        match s.bytes().next().ok_or(Error::InvalidCellType)? {
            b's' => Ok(CellType::String(default.unwrap_or_default().to_string())),
            b'b' => Ok(CellType::Boolean(matches!(
                default.unwrap_or_default(),
                "true" | "1"
            ))),
            b'i' => Ok(CellType::Integer(
                u32::from_str(default.unwrap_or_default()).unwrap_or_default(),
            )),
            b'e' => Ok(CellType::Enum(
                options.unwrap_or_default(),
                usize::from_str(default.unwrap_or_default()).unwrap_or_default(),
            )),
            _ => Err(Error::UnknownCellDatatype),
        }
    }
}

#[binread]
#[derive(Debug)]
#[br(magic = b"DTIIFORM")]
pub struct DataTable {
    pub size: u32,
    #[br( count = 4, try_map = |data: Vec<u8>| String::from_utf8(data) )]
    pub version: String,

    // Columns
    #[br(magic = b"COLS", temp)]
    pub _columns_size: u32,
    #[br(pad_after = 3)]
    pub columns_count: u8,
    #[br(count = columns_count)]
    pub columns: Vec<NullString>,

    // // Types
    #[br(magic = b"TYPE", temp)]
    pub _type_size: u32,
    #[br(count = columns_count, try_map = |a: Vec<NullString>| a.iter().map(CellType::try_from).collect::<Result<Vec<CellType>, Error>>())]
    pub types: Vec<CellType>,

    // // Rows
    #[br(magic = b"ROWS", parse_with = binrw::helpers::read_u24)]
    pub row_count: u32,
    pub _row_size: u32,
    #[br(count = row_count, args{ inner: (&columns, &types)})]
    pub rows: Vec<Row>,
}

impl TryFrom<crate::iff::IFFFile> for DataTable {
    type Error = binrw::Error;

    fn try_from(value: crate::iff::IFFFile) -> Result<Self, Self::Error> {
        DataTable::read_be(&mut Cursor::new(value.data))
    }
}
