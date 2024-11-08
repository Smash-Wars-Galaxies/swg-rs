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

    #[error("{0}")]
    WinnowError(winnow::error::ErrMode<winnow::error::ContextError>),

    #[error("Invalid column header")]
    InvalidColumnHeader,

    #[error("Invalid cell type")]
    InvalidCellType,

    #[error("Unknown cell datatype")]
    UnknownCellDatatype,
}

/// Generic result type with crate's Error as its error variant
pub type Result<T> = core::result::Result<T, Error>;
