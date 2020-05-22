#[derive(Debug)]
pub(super) enum EncoderError {
    CodelenError,
    IO(std::io::Error),
}

use std::fmt;
impl fmt::Display for EncoderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            EncoderError::CodelenError => write!(f, "Codeword too long"),
            EncoderError::IO(ref err) => err.fmt(f),
        }
    }
}

use std::error::Error;
impl Error for EncoderError {}

impl std::convert::From<std::io::Error> for EncoderError {
    fn from(err: std::io::Error) -> Self {
        EncoderError::IO(err)
    }
}

impl std::convert::Into<std::io::Error> for EncoderError {
    fn into(self) -> std::io::Error {
        match self {
            EncoderError::IO(err) => err,
            _ => std::io::Error::new(std::io::ErrorKind::Other, self),
        }
    }
}
