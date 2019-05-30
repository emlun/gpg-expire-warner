#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    TimeError(std::time::SystemTimeError),
    TomlError(toml::de::Error),
    Utf8Error(std::string::FromUtf8Error),
}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::IoError(e)
    }
}
impl From<std::time::SystemTimeError> for Error {
    fn from(e: std::time::SystemTimeError) -> Error {
        Error::TimeError(e)
    }
}
impl From<toml::de::Error> for Error {
    fn from(e: toml::de::Error) -> Error {
        Error::TomlError(e)
    }
}
impl From<std::string::FromUtf8Error> for Error {
    fn from(e: std::string::FromUtf8Error) -> Error {
        Error::Utf8Error(e)
    }
}
