use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Diagnostic, Debug)]
pub enum Error {
    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error(transparent)]
    BinRWError(#[from] binrw::Error),

    #[error(transparent)]
    Utf8Error(#[from] std::str::Utf8Error),

    #[error("Invalid column header")]
    InvalidColumnHeader,

    #[error("Invalid cell type")]
    InvalidCellType,

    #[error("Unknown cell datatype")]
    UnknownCellDatatype,
}
