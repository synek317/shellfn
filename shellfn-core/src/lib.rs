use std::str::FromStr;
use std::io::{self, BufReader, BufRead};
use std::process::{Command, Stdio, Child};
use std::ffi::OsStr;
use itertools::Either;
use std::error::Error as StdError;
mod error;

pub use error::*;

pub fn execute_void_nopanic<TArg, TEnvKey, TEnvVal>(
    cmd:  impl AsRef<OsStr>,
    args: impl IntoIterator<Item = TArg>,
    envs: impl IntoIterator<Item = (TEnvKey, TEnvVal)>
)
    where TArg: AsRef<OsStr>,
          TEnvKey: AsRef<OsStr>,
          TEnvVal: AsRef<OsStr>,
{
    let _ = spawn(cmd, args, envs)
        .and_then(Child::wait_with_output);
}

pub fn execute_void_panic<TArg, TEnvKey, TEnvVal>(
    cmd:  impl AsRef<OsStr>,
    args: impl IntoIterator<Item = TArg>,
    envs: impl IntoIterator<Item = (TEnvKey, TEnvVal)>
)
    where TArg: AsRef<OsStr>,
          TEnvKey: AsRef<OsStr>,
          TEnvVal: AsRef<OsStr>,
{
    let output = spawn(cmd, args, envs)
        .and_then(Child::wait_with_output)
        .expect("Shell execution failed");

    if !output.status.success() {
          panic!("Shell execution failed")
    }
}

pub fn execute_parse_result<T, TArg, TEnvKey, TEnvVal, TError>(
    cmd:  impl AsRef<OsStr>,
    args: impl IntoIterator<Item = TArg>,
    envs: impl IntoIterator<Item = (TEnvKey, TEnvVal)>
) -> Result<T, TError>
    where T: FromStr,
          TArg: AsRef<OsStr>,
          TEnvKey: AsRef<OsStr>,
          TEnvVal: AsRef<OsStr>,
          <T as FromStr>::Err: StdError,
          TError: From<Error<<T as FromStr>::Err>>,
{
    let process = spawn(cmd, args, envs)
        .map_err(Error::ProcessNotSpawned)?;

    let result = process
        .wait_with_output()
        .map_err(Error::WaitFailed)?;

    if !result.status.success() {
        return Err(Error::ProcessFailed(result))?;
    }

    String::from_utf8(result.stdout)
        .map_err(Error::NonUtf8Stdout)
        .map_err(Into::into)
        .and_then(|s| s.parse().map_err(Error::ParsingError).map_err(Into::into))
}

pub fn execute_parse_panic<T, TArg, TEnvKey, TEnvVal>(
    cmd:  impl AsRef<OsStr>,
    args: impl IntoIterator<Item = TArg>,
    envs: impl IntoIterator<Item = (TEnvKey, TEnvVal)>
) -> T
    where T: FromStr,
          TArg: AsRef<OsStr>,
          TEnvKey: AsRef<OsStr>,
          TEnvVal: AsRef<OsStr>,
          <T as FromStr>::Err: StdError,
{
    let result = spawn(cmd, args, envs)
        .expect("Shell execution failed")
        .wait_with_output()
        .expect("Shell execution failed");

    if !result.status.success() {
        panic!("Shell execution failed");
    }

    String::from_utf8(result.stdout)
        .expect("Shell execution failed")
        .parse()
        .expect("Shell execution failed")
}

pub fn execute_iter_result_result<T, TArg, TEnvKey, TEnvVal, TOuterError, TInnerError>(
    cmd:  impl AsRef<OsStr>,
    args: impl IntoIterator<Item=TArg>,
    envs: impl IntoIterator<Item=(TEnvKey, TEnvVal)>
) -> Result<impl Iterator<Item=Result<T, TInnerError>>, TOuterError>
    where T: FromStr,
          TArg: AsRef<OsStr>,
          TEnvKey: AsRef<OsStr>,
          TEnvVal: AsRef<OsStr>,
          <T as FromStr>::Err: StdError,
          TOuterError: From<Error<<T as FromStr>::Err>>,
          TInnerError: From<Error<<T as FromStr>::Err>>,
{
    let mut process = spawn(cmd, args, envs)
        .map_err(Error::ProcessNotSpawned)?;

    let stdout = process.stdout.take().unwrap();

    Ok(BufReader::new(stdout)
        .lines()
        .map(|r| r
            .map_err(Error::StdoutUnreadable)
            .map_err(Into::into)
            .and_then(|line| line.parse().map_err(Error::ParsingError).map_err(Into::into))
        )
    )
}

pub fn execute_iter_panic_panic<T, TArg, TEnvKey, TEnvVal>(
    cmd:  impl AsRef<OsStr>,
    args: impl IntoIterator<Item=TArg>,
    envs: impl IntoIterator<Item=(TEnvKey, TEnvVal)>
) -> impl Iterator<Item=T>
    where T: FromStr,
          TArg: AsRef<OsStr>,
          TEnvKey: AsRef<OsStr>,
          TEnvVal: AsRef<OsStr>,
          <T as FromStr>::Err: StdError,
{
    let mut process = spawn(cmd, args, envs)
        .expect("Shell execution failed");
    let stdout = process
        .stdout
        .take()
        .unwrap();

    BufReader::new(stdout)
        .lines()
        .map(|r| r
            .expect("Shell execution failed")
            .parse()
            .expect("Shell execution failed")
        )
        .chain([()].into_iter().flat_map(move |_| { if !process.wait().unwrap().success() { panic!("Foo") } std::iter::empty() }))
}

pub fn execute_iter_panic_result<T, TArg, TEnvKey, TEnvVal, TError>(
    cmd:  impl AsRef<OsStr>,
    args: impl IntoIterator<Item=TArg>,
    envs: impl IntoIterator<Item=(TEnvKey, TEnvVal)>
) -> impl Iterator<Item=Result<T, TError>>
    where T: FromStr,
          TArg: AsRef<OsStr>,
          TEnvKey: AsRef<OsStr>,
          TEnvVal: AsRef<OsStr>,
          <T as FromStr>::Err: StdError,
          TError: From<Error<<T as FromStr>::Err>>,
{
    let mut process = spawn(cmd, args, envs)
        .expect("Shell execution failed");
    let stdout = process
        .stdout
        .take()
        .unwrap();

    BufReader::new(stdout)
        .lines()
        .map(|r| r
            .map_err(Error::StdoutUnreadable)
            .map_err(Into::into)
            .and_then(|line| line.parse().map_err(Error::ParsingError).map_err(Into::into))
        )
        .chain([()].into_iter().flat_map(move |_| { if !process.wait().unwrap().success() { panic!("Shell execution failed") } std::iter::empty() }))
}

pub fn execute_iter_nopanic_result<T, TArg, TEnvKey, TEnvVal, TError>(
    cmd:  impl AsRef<OsStr>,
    args: impl IntoIterator<Item=TArg>,
    envs: impl IntoIterator<Item=(TEnvKey, TEnvVal)>
) -> impl Iterator<Item=Result<T, TError>>
    where T: FromStr,
          TArg: AsRef<OsStr>,
          TEnvKey: AsRef<OsStr>,
          TEnvVal: AsRef<OsStr>,
          <T as FromStr>::Err: StdError,
          TError: From<Error<<T as FromStr>::Err>>,
{
    spawn(cmd, args, envs)
        .ok()
        .map(|mut process|
            BufReader::new(process.stdout.take().unwrap())
                .lines()
                .map(|r| r
                    .map_err(Error::StdoutUnreadable)
                    .map_err(Into::into)
                    .and_then(|line| line.parse().map_err(Error::ParsingError).map_err(Into::into))
                )
        )
    .map_or_else(
        || Either::Right(std::iter::empty()),
        |iter| Either::Left(iter)
    )
}

pub fn execute_iter_nopanic_nopanic<T, TArg, TEnvKey, TEnvVal>(
    cmd:  impl AsRef<OsStr>,
    args: impl IntoIterator<Item=TArg>,
    envs: impl IntoIterator<Item=(TEnvKey, TEnvVal)>
) -> impl Iterator<Item=T>
    where T: FromStr,
          TArg: AsRef<OsStr>,
          TEnvKey: AsRef<OsStr>,
          TEnvVal: AsRef<OsStr>,
          <T as FromStr>::Err: StdError
{
    spawn(cmd, args, envs)
        .ok()
        .map(|mut process|
            BufReader::new(process.stdout.take().unwrap())
                .lines()
                .filter_map(|r| r.ok().and_then(|line| line.parse().ok()))
        )
    .map_or_else(
        || Either::Right(std::iter::empty()),
        |iter| Either::Left(iter)
    )
}

pub fn execute_iter_result_panic<T, TArg, TEnvKey, TEnvVal, TError>(
    cmd:  impl AsRef<OsStr>,
    args: impl IntoIterator<Item=TArg>,
    envs: impl IntoIterator<Item=(TEnvKey, TEnvVal)>
) -> Result<impl Iterator<Item=T>, TError>
    where T: FromStr,
          TArg: AsRef<OsStr>,
          TEnvKey: AsRef<OsStr>,
          TEnvVal: AsRef<OsStr>,
          <T as FromStr>::Err: StdError,
          TError: From<Error<<T as FromStr>::Err>>,
{
    let mut process = spawn(cmd, args, envs)
        .map_err(Error::ProcessNotSpawned)?;

    let stdout = process.stdout.take().unwrap();

    Ok(BufReader::new(stdout)
        .lines()
        .map(|r| r
            .expect("Shell execution failed")
            .parse()
            .expect("Shell execution failed")
        )
    )
}

pub fn execute_iter_result_nopanic<T, TArg, TEnvKey, TEnvVal, TError>(
    cmd:  impl AsRef<OsStr>,
    args: impl IntoIterator<Item=TArg>,
    envs: impl IntoIterator<Item=(TEnvKey, TEnvVal)>
) -> Result<impl Iterator<Item=T>, TError>
    where T: FromStr,
          TArg: AsRef<OsStr>,
          TEnvKey: AsRef<OsStr>,
          TEnvVal: AsRef<OsStr>,
          <T as FromStr>::Err: StdError,
          TError: From<Error<<T as FromStr>::Err>>,
{
    let mut process = spawn(cmd, args, envs)
        .map_err(Error::ProcessNotSpawned)?;

    let stdout = process.stdout.take().unwrap();

    Ok(BufReader::new(stdout)
        .lines()
        .filter_map(|r| r
            .ok()
            .and_then(|item| item.parse().ok())
        )
    )
}


fn spawn<TArg, TEnvKey, TEnvVal>(
    cmd:  impl AsRef<OsStr>,
    args: impl IntoIterator<Item = TArg>,
    envs: impl IntoIterator<Item = (TEnvKey, TEnvVal)>
) -> Result<Child, io::Error>
    where TArg: AsRef<OsStr>,
          TEnvKey: AsRef<OsStr>,
          TEnvVal: AsRef<OsStr>,
{
    Command::new(cmd)
        .stdout(Stdio::piped())
        .args(args)
        .envs(envs)
        .spawn()
}
