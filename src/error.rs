#[derive(Debug)]
pub enum Error {
    TimeError(std::time::SystemTimeError),
    Utf8Error(std::string::FromUtf8Error),
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
