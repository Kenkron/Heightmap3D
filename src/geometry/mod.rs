pub mod triangle;
pub mod heightmap;
use std::num::{ParseFloatError, ParseIntError};
use image::ImageError;

#[derive(Debug, Clone)]
pub enum ReadError {
    IO,
    ParseFloat,
    ParseInt,
    Image
}
impl From<ParseIntError> for ReadError {
    fn from(_e: ParseIntError) -> Self {Self::ParseInt}
}
impl From<ParseFloatError> for ReadError {
    fn from(_e: ParseFloatError) -> Self {Self::ParseFloat}
}
impl From<std::io::Error> for ReadError {
    fn from(_e: std::io::Error) -> Self {Self::IO}
}
impl From<ImageError> for ReadError {
    fn from(_e: ImageError) -> Self {Self::Image}
}
impl std::fmt::Display for ReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Self::IO => write!(f, "Read/Write Error"),
            Self::ParseFloat => write!(f, "Parse float Error"),
            Self::ParseInt => write!(f, "Parse int Error"),
            Self::Image => write!(f, "Image Error")
        }
    }
}
impl std::error::Error for ReadError {}