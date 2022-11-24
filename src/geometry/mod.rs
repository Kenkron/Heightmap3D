pub mod triangle;
pub mod heightmap;
use std::num::{ParseFloatError, ParseIntError};

#[derive(Debug, Clone)]
pub enum ReadError {
    IOError,
    ParseFloatError,
    ParseIntError
}
impl From<ParseIntError> for ReadError {
    fn from(_e: ParseIntError) -> Self {Self::ParseIntError}
}
impl From<ParseFloatError> for ReadError {
    fn from(_e: ParseFloatError) -> Self {Self::ParseFloatError}
}
impl From<std::io::Error> for ReadError {
    fn from(_e: std::io::Error) -> Self {Self::IOError}
}
impl std::fmt::Display for ReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Self::IOError => write!(f, "Read/Write Error"),
            Self::ParseFloatError => write!(f, "Parse float Error"),
            Self::ParseIntError => write!(f, "Parse int Error")
        }
    }
}
impl std::error::Error for ReadError {}