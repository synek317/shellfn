use std::error::Error as StdError;
use std::io;
use std::process::Output;
use std::string::FromUtf8Error;

#[derive(Debug)]
pub enum Error<PE: StdError> {
    NonUtf8Stdout(FromUtf8Error),
    ParsingError(PE),
    ProcessNotSpawned(io::Error),
    StdoutUnreadable(io::Error),
    WaitFailed(io::Error),
    ProcessFailed(Output),
}

impl<PE: StdError> StdError for Error<PE> {
    fn description(&self) -> &str {
        match self {
            Error::NonUtf8Stdout(_)     => "subprocess stdout contains non-utf8 characters",
            Error::ParsingError(_)      => "could not parse subprocess output",
            Error::ProcessNotSpawned(_) => "could not spawn subprocess",
            Error::StdoutUnreadable(_)  => "could not read subprocess stdout",
            Error::WaitFailed(_)        => "subprocess failed",
            Error::ProcessFailed(_)     => "subprocess finished with error",
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match self {
            Error::NonUtf8Stdout(ref e)     => Some(e),
            Error::ParsingError(ref e)      => Some(e),
            Error::ProcessNotSpawned(ref e) => Some(e),
            Error::StdoutUnreadable(ref e)  => Some(e),
            Error::WaitFailed(ref e)        => Some(e),
            Error::ProcessFailed(_)         => None,
        }
    }
}

impl<PE: StdError> std::fmt::Display for Error<PE> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}
