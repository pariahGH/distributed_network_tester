use std::io::Error;
use std::io;
use std::ffi::OsString;

#[derive(Debug)]
pub enum ClientError {
    IOError(io::Error),
    ClientError(String)
}

impl From<Error> for ClientError {
    fn from(err: Error) -> ClientError {
        ClientError::IOError(err)
    }
}

impl From<String> for ClientError {
    fn from(err: String) -> ClientError {
        ClientError::ClientError(err)
    }
}

impl From<OsString> for ClientError {
    fn from(err: OsString) -> ClientError {
        ClientError::ClientError(err.into_string().unwrap())
    }
}

pub type ClientResult<T, ClientError> = Result<T, ClientError>;