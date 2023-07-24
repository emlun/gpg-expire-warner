#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    ParseIntError(std::num::ParseIntError),
    TimeError(std::time::SystemTimeError),
    Utf8Error(std::string::FromUtf8Error),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Io(e) => write!(f, "{}", e),
            Self::ParseIntError(e) => write!(f, "{}", e),
            Self::TimeError(e) => write!(f, "{}", e),
            Self::Utf8Error(e) => write!(f, "{}", e),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::Io(e)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(e: std::num::ParseIntError) -> Error {
        Error::ParseIntError(e)
    }
}

impl From<std::time::SystemTimeError> for Error {
    fn from(e: std::time::SystemTimeError) -> Error {
        Error::TimeError(e)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(e: std::string::FromUtf8Error) -> Error {
        Error::Utf8Error(e)
    }
}
