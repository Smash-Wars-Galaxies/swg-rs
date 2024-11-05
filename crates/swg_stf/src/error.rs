//! Error types that can be emitted from this library
//!

use miette::Diagnostic;
use thiserror::Error;

/// Error type for library
#[derive(Error, Diagnostic, Debug)]
pub enum Error {
    /// Transparent warpper for [`std::io::Error`]
    #[error(transparent)]
    IOError(#[from] std::io::Error),

    /// Transparent warpper for [`std::string::FromUtf8Error`]
    #[error(transparent)]
    UTF8Error(#[from] std::string::FromUtf8Error),

    /// Transparent warpper for [`std::string::FromUtf16Error`]
    #[error(transparent)]
    UTF16Error(#[from] std::string::FromUtf16Error),

    /// File is an invalid string table file
    #[error("Invalid String Table")]
    InvalidFile,
}

/// Generic result type with crate's Error as its error variant
pub type Result<T> = core::result::Result<T, Error>;
