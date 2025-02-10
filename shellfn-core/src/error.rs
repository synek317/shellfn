use std::error::Error as StdError;
use std::io;
use std::process::Output;
use std::string::FromUtf8Error;

#[derive(thiserror::Error, Debug)]
pub enum Error<PE: StdError> {
    #[error("subprocess stdout contains non-utf8 characters")]
    NonUtf8Stdout(#[source] FromUtf8Error),
    #[error("could not parse subprocess output")]
    ParsingError(#[source] PE),
    #[error("could not spawn subprocess")]
    ProcessNotSpawned(#[source] io::Error),
    #[error("could not read subprocess stdout")]
    StdoutUnreadable(#[source] io::Error),
    #[error("subprocess failed")]
    WaitFailed(#[source] io::Error),
    #[error("subprocess finished with error")]
    ProcessFailed(Output),
}

// TODO: replace with `!` after stabilization
#[derive(thiserror::Error, Debug, Copy, Clone, Eq, PartialEq)]
pub enum NeverError {}
