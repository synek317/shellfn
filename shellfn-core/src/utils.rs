use crate::error::Error;
use std::error::Error as StdError;
use std::ffi::OsStr;
use std::io;
use std::process::{Child, Command, Stdio};

pub const PANIC_MSG: &str = "Shell execution failed";

pub fn spawn<TArg, TEnvKey, TEnvVal>(
    cmd: impl AsRef<OsStr>,
    args: impl IntoIterator<Item = TArg>,
    envs: impl IntoIterator<Item = (TEnvKey, TEnvVal)>,
) -> Result<Child, io::Error>
where
    TArg: AsRef<OsStr>,
    TEnvKey: AsRef<OsStr>,
    TEnvVal: AsRef<OsStr>,
{
    Command::new(cmd)
        .stdout(Stdio::piped())
        .args(args)
        .envs(envs)
        .spawn()
}

pub fn check_exit_code<E: StdError>(process: Child) -> Result<(), Error<E>> {
    let output = process.wait_with_output().map_err(Error::WaitFailed)?;

    if !output.status.success() {
        Err(Error::ProcessFailed(output))
    } else {
        Ok(())
    }
}

pub fn check_exit_code_panic(process: Child) {
    let output = process.wait_with_output().expect(PANIC_MSG);

    if !output.status.success() {
            panic!("{}", PANIC_MSG)
    }
}