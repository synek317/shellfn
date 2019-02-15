use crate::error::Error;
use crate::utils::spawn;
use itertools::Either;
use std::error::Error as StdError;
use std::ffi::OsStr;
use std::io::{BufRead, BufReader};
use std::process::Child;
use std::str::FromStr;

const PANIC_MSG: &'static str = "Shell execution failed";

pub fn execute_void_nopanic<TArg, TEnvKey, TEnvVal>(
    cmd:  impl AsRef<OsStr>,
    args: impl IntoIterator<Item = TArg>,
    envs: impl IntoIterator<Item = (TEnvKey, TEnvVal)>,
) where
    TArg: AsRef<OsStr>,
    TEnvKey: AsRef<OsStr>,
    TEnvVal: AsRef<OsStr>,
{
    let _ = spawn(cmd, args, envs).and_then(Child::wait_with_output);
}

pub fn execute_void_panic<TArg, TEnvKey, TEnvVal>(
    cmd:  impl AsRef<OsStr>,
    args: impl IntoIterator<Item = TArg>,
    envs: impl IntoIterator<Item = (TEnvKey, TEnvVal)>,
) where
    TArg: AsRef<OsStr>,
    TEnvKey: AsRef<OsStr>,
    TEnvVal: AsRef<OsStr>,
{
    let output = spawn(cmd, args, envs)
        .and_then(Child::wait_with_output)
        .expect(PANIC_MSG);

    if !output.status.success() {
        panic!(PANIC_MSG)
    }
}

pub fn execute_parse_result<T, TArg, TEnvKey, TEnvVal, TError>(
    cmd:  impl AsRef<OsStr>,
    args: impl IntoIterator<Item = TArg>,
    envs: impl IntoIterator<Item = (TEnvKey, TEnvVal)>,
) -> Result<T, TError>
where
    T: FromStr,
    TArg: AsRef<OsStr>,
    TEnvKey: AsRef<OsStr>,
    TEnvVal: AsRef<OsStr>,
    <T as FromStr>::Err: StdError,
    TError: From<Error<<T as FromStr>::Err>>,
{
    let process = spawn(cmd, args, envs).map_err(Error::ProcessNotSpawned)?;
    let result  = process.wait_with_output().map_err(Error::WaitFailed)?;

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
    envs: impl IntoIterator<Item = (TEnvKey, TEnvVal)>,
) -> T
where
    T: FromStr,
    TArg: AsRef<OsStr>,
    TEnvKey: AsRef<OsStr>,
    TEnvVal: AsRef<OsStr>,
    <T as FromStr>::Err: StdError,
{
    let result = spawn(cmd, args, envs)
        .expect(PANIC_MSG)
        .wait_with_output()
        .expect(PANIC_MSG);

    if !result.status.success() {
        panic!(PANIC_MSG);
    }

    String::from_utf8(result.stdout)
        .expect(PANIC_MSG)
        .parse()
        .expect(PANIC_MSG)
}

pub fn execute_iter_result_result<T, TArg, TEnvKey, TEnvVal, TOuterError, TInnerError>(
    cmd:  impl AsRef<OsStr>,
    args: impl IntoIterator<Item = TArg>,
    envs: impl IntoIterator<Item = (TEnvKey, TEnvVal)>,
) -> Result<impl Iterator<Item = Result<T, TInnerError>>, TOuterError>
where
    T: FromStr,
    TArg: AsRef<OsStr>,
    TEnvKey: AsRef<OsStr>,
    TEnvVal: AsRef<OsStr>,
    <T as FromStr>::Err: StdError,
    TOuterError: From<Error<<T as FromStr>::Err>>,
    TInnerError: From<Error<<T as FromStr>::Err>>,
{
    let mut process = spawn(cmd, args, envs).map_err(Error::ProcessNotSpawned)?;
    let stdout      = process.stdout.take().unwrap();

    Ok(BufReader::new(stdout).lines().map(|lres| {
        lres.map_err(Error::StdoutUnreadable)
            .map_err(Into::into)
            .and_then(|line| {
                line.parse()
                    .map_err(Error::ParsingError)
                    .map_err(Into::into)
            })
    }))
}

pub fn execute_iter_panic_panic<T, TArg, TEnvKey, TEnvVal>(
    cmd:  impl AsRef<OsStr>,
    args: impl IntoIterator<Item = TArg>,
    envs: impl IntoIterator<Item = (TEnvKey, TEnvVal)>,
) -> impl Iterator<Item = T>
where
    T: FromStr,
    TArg: AsRef<OsStr>,
    TEnvKey: AsRef<OsStr>,
    TEnvVal: AsRef<OsStr>,
    <T as FromStr>::Err: StdError,
{
    let mut process = spawn(cmd, args, envs).expect(PANIC_MSG);
    let stdout      = process.stdout.take().unwrap();

    BufReader::new(stdout)
        .lines()
        .map(|lres| {
            lres.expect(PANIC_MSG)
                .parse()
                .expect(PANIC_MSG)
        })
        .chain([()].into_iter().flat_map(move |_| {
            if !process.wait().unwrap().success() {
                panic!(PANIC_MSG)
            }
            std::iter::empty()
        }))
}

pub fn execute_iter_panic_result<T, TArg, TEnvKey, TEnvVal, TError>(
    cmd:  impl AsRef<OsStr>,
    args: impl IntoIterator<Item = TArg>,
    envs: impl IntoIterator<Item = (TEnvKey, TEnvVal)>,
) -> impl Iterator<Item = Result<T, TError>>
where
    T: FromStr,
    TArg: AsRef<OsStr>,
    TEnvKey: AsRef<OsStr>,
    TEnvVal: AsRef<OsStr>,
    <T as FromStr>::Err: StdError,
    TError: From<Error<<T as FromStr>::Err>>,
{
    let mut process = spawn(cmd, args, envs).expect(PANIC_MSG);
    let stdout      = process.stdout.take().unwrap();

    BufReader::new(stdout)
        .lines()
        .map(|lres| {
            lres.map_err(Error::StdoutUnreadable)
                .map_err(Into::into)
                .and_then(|line| {
                    line.parse()
                        .map_err(Error::ParsingError)
                        .map_err(Into::into)
                })
        })
        .chain([()].into_iter().flat_map(move |_| {
            if !process.wait().unwrap().success() {
                panic!(PANIC_MSG)
            }
            std::iter::empty()
        }))
}

pub fn execute_iter_nopanic_result<T, TArg, TEnvKey, TEnvVal, TError>(
    cmd:  impl AsRef<OsStr>,
    args: impl IntoIterator<Item = TArg>,
    envs: impl IntoIterator<Item = (TEnvKey, TEnvVal)>,
) -> impl Iterator<Item = Result<T, TError>>
where
    T: FromStr,
    TArg: AsRef<OsStr>,
    TEnvKey: AsRef<OsStr>,
    TEnvVal: AsRef<OsStr>,
    <T as FromStr>::Err: StdError,
    TError: From<Error<<T as FromStr>::Err>>,
{
    spawn(cmd, args, envs)
        .ok()
        .map(|mut process| {
            BufReader::new(process.stdout.take().unwrap())
                .lines()
                .map(|lres| {
                    lres.map_err(Error::StdoutUnreadable)
                        .map_err(Into::into)
                        .and_then(|line| {
                            line.parse()
                                .map_err(Error::ParsingError)
                                .map_err(Into::into)
                        })
                })
        })
        .map_or_else(
            || Either::Right(std::iter::empty()),
            |iter| Either::Left(iter),
        )
}

pub fn execute_iter_nopanic_nopanic<T, TArg, TEnvKey, TEnvVal>(
    cmd:  impl AsRef<OsStr>,
    args: impl IntoIterator<Item = TArg>,
    envs: impl IntoIterator<Item = (TEnvKey, TEnvVal)>,
) -> impl Iterator<Item = T>
where
    T: FromStr,
    TArg: AsRef<OsStr>,
    TEnvKey: AsRef<OsStr>,
    TEnvVal: AsRef<OsStr>,
    <T as FromStr>::Err: StdError,
{
    spawn(cmd, args, envs)
        .ok()
        .map(|mut process| {
            BufReader::new(process.stdout.take().unwrap())
                .lines()
                .filter_map(|lres| lres.ok().and_then(|line| line.parse().ok()))
        })
        .map_or_else(
            || Either::Right(std::iter::empty()),
            |iter| Either::Left(iter),
        )
}

pub fn execute_iter_result_panic<T, TArg, TEnvKey, TEnvVal, TError>(
    cmd:  impl AsRef<OsStr>,
    args: impl IntoIterator<Item = TArg>,
    envs: impl IntoIterator<Item = (TEnvKey, TEnvVal)>,
) -> Result<impl Iterator<Item = T>, TError>
where
    T: FromStr,
    TArg: AsRef<OsStr>,
    TEnvKey: AsRef<OsStr>,
    TEnvVal: AsRef<OsStr>,
    <T as FromStr>::Err: StdError,
    TError: From<Error<<T as FromStr>::Err>>,
{
    let mut process = spawn(cmd, args, envs).map_err(Error::ProcessNotSpawned)?;
    let stdout      = process.stdout.take().unwrap();

    Ok(BufReader::new(stdout).lines().map(|lres| {
        lres.expect(PANIC_MSG)
            .parse()
            .expect(PANIC_MSG)
    }))
}

pub fn execute_iter_result_nopanic<T, TArg, TEnvKey, TEnvVal, TError>(
    cmd:  impl AsRef<OsStr>,
    args: impl IntoIterator<Item = TArg>,
    envs: impl IntoIterator<Item = (TEnvKey, TEnvVal)>,
) -> Result<impl Iterator<Item = T>, TError>
where
    T: FromStr,
    TArg: AsRef<OsStr>,
    TEnvKey: AsRef<OsStr>,
    TEnvVal: AsRef<OsStr>,
    <T as FromStr>::Err: StdError,
    TError: From<Error<<T as FromStr>::Err>>,
{
    let mut process = spawn(cmd, args, envs).map_err(Error::ProcessNotSpawned)?;
    let stdout      = process.stdout.take().unwrap();

    Ok(BufReader::new(stdout)
        .lines()
        .filter_map(|lres| lres.ok().and_then(|item| item.parse().ok())))
}
