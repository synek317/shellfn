use crate::error::Error;
use crate::execute::execute_iter_nopanic_nopanic;
use crate::utils::*;
use std::error::Error as StdError;
use std::ffi::OsStr;
use std::io::{BufRead, BufReader};
use std::str::FromStr;

/// Executes command with args and environment variables, parses output line by line, returns after reading whole output
/// * On invalid command: return error
/// * On error exit code: return error
/// * On parsing failure: collect error item
/// * Possible errors: ProcessNotSpawned, WaitFailed, ProcessFailed, StdoutUnreadable (item error), ParsingError (item error)
///
/// Designed for
/// ```rust
/// use shellfn::shell;
/// use std::error::Error;
///
/// #[shell]
/// fn command() -> Result<Vec<Result<u32, Box<Error + 'static>>>, Box<Error>> {
///     "echo 1; echo 2; echo 3"
/// }
///
/// assert_eq!(vec![1, 2, 3], command().unwrap().map(Result::unwrap).collect::<Vec<_>>())
/// ```
pub fn execute_vec_result_result<T, TArg, TEnvKey, TEnvVal, TOuterError, TInnerError>(
    cmd: impl AsRef<OsStr>,
    args: impl IntoIterator<Item = TArg>,
    envs: impl IntoIterator<Item = (TEnvKey, TEnvVal)>,
) -> Result<Vec<Result<T, TInnerError>>, TOuterError>
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
    let stdout = process.stdout.take().unwrap();
    let result = BufReader::new(stdout)
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
        .collect::<Vec<_>>();

    check_exit_code(process)?;
    Ok(result)
}

/// Executes command with args and environment variables, parses output line by line, returns after reading whole output
/// * On invalid command: panic
/// * On error exit code: panic
/// * On parsing failure: panic
/// * Possible errors: N/A
///
/// Designed for
/// ```rust
/// use shellfn::shell;
/// use std::error::Error;
///
/// #[shell]
/// fn command() -> Vec<u32> {
///     "echo 1; echo 2; echo 3"
/// }
///
/// assert_eq!(vec![1, 2, 3], command().collect::<Vec<_>>())
/// ```
pub fn execute_vec_panic_panic<T, TArg, TEnvKey, TEnvVal>(
    cmd: impl AsRef<OsStr>,
    args: impl IntoIterator<Item = TArg>,
    envs: impl IntoIterator<Item = (TEnvKey, TEnvVal)>,
) -> Vec<T>
where
    T: FromStr,
    TArg: AsRef<OsStr>,
    TEnvKey: AsRef<OsStr>,
    TEnvVal: AsRef<OsStr>,
    <T as FromStr>::Err: StdError,
{
    let mut process = spawn(cmd, args, envs).expect(PANIC_MSG);
    let stdout = process.stdout.take().unwrap();
    let result = BufReader::new(stdout)
        .lines()
        .map(|lres| lres.expect(PANIC_MSG).parse().expect(PANIC_MSG))
        .collect::<Vec<_>>();

    check_exit_code_panic(process);
    result
}

/// Executes command with args and environment variables, parses output line by line, returns after reading whole output
/// * On invalid command: panic
/// * On error exit code: panic
/// * On parsing failure: collect error item
/// * Possible errors: StdoutUnreadable (item error), ParsingError (item error)
///
/// Designed for
/// ```rust
/// use shellfn::shell;
/// use std::error::Error;
///
/// #[shell]
/// fn command() -> Vec<Result<u32, Box<Error + 'static>>> {
///     "echo 1; echo 2; echo 3"
/// }
///
/// assert_eq!(vec![1, 2, 3], command().map(Result::unwrap).collect::<Vec<_>>())
/// ```
pub fn execute_vec_panic_result<T, TArg, TEnvKey, TEnvVal, TError>(
    cmd: impl AsRef<OsStr>,
    args: impl IntoIterator<Item = TArg>,
    envs: impl IntoIterator<Item = (TEnvKey, TEnvVal)>,
) -> Vec<Result<T, TError>>
where
    T: FromStr,
    TArg: AsRef<OsStr>,
    TEnvKey: AsRef<OsStr>,
    TEnvVal: AsRef<OsStr>,
    <T as FromStr>::Err: StdError,
    TError: From<Error<<T as FromStr>::Err>>,
{
    let mut process = spawn(cmd, args, envs).expect(PANIC_MSG);
    let stdout = process.stdout.take().unwrap();
    let result = BufReader::new(stdout)
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
        .collect::<Vec<_>>();

    check_exit_code_panic(process);
    result
}

/// Executes command with args and environment variables, parses output line by line, returns after reading whole output
/// * On invalid command: return empty vec
/// * On error exit code: return already collected items
/// * On parsing failure: collect error item
/// * Possible errors: StdoutUnreadable (item error), ParsingError (item error)
///
/// Designed for
/// ```rust
/// use shellfn::shell;
/// use std::error::Error;
///
/// #[shell(no_panic)]
/// fn command() -> Vec<Result<u32, Box<Error + 'static>>> {
///     "echo 1; echo 2; echo 3"
/// }
///
/// assert_eq!(vec![1, 2, 3], command().map(Result::unwrap).collect::<Vec<_>>())
/// ```
pub fn execute_vec_nopanic_result<T, TArg, TEnvKey, TEnvVal, TError>(
    cmd: impl AsRef<OsStr>,
    args: impl IntoIterator<Item = TArg>,
    envs: impl IntoIterator<Item = (TEnvKey, TEnvVal)>,
) -> Vec<Result<T, TError>>
where
    T: FromStr,
    TArg: AsRef<OsStr>,
    TEnvKey: AsRef<OsStr>,
    TEnvVal: AsRef<OsStr>,
    <T as FromStr>::Err: StdError,
    TError: From<Error<<T as FromStr>::Err>>,
{
    spawn(cmd, args, envs)
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
                .collect::<Vec<_>>()
        })
        .unwrap_or(Vec::default())
}

/// Executes command with args and environment variables, parses output line by line, returns after reading whole output
/// * On invalid command: return empty vec
/// * On error exit code: return already collected items
/// * On parsing failure: skip item
/// * Possible errors: N/A
///
/// Designed for
/// ```rust
/// use shellfn::shell;
/// use std::error::Error;
///
/// #[shell(no_panic)]
/// fn command() -> Vec<u32> {
///     "echo 1; echo 2; echo 3"
/// }
///
/// assert_eq!(vec![1, 2, 3], command().collect::<Vec<_>>())
/// ```
pub fn execute_vec_nopanic_nopanic<T, TArg, TEnvKey, TEnvVal>(
    cmd: impl AsRef<OsStr>,
    args: impl IntoIterator<Item = TArg>,
    envs: impl IntoIterator<Item = (TEnvKey, TEnvVal)>,
) -> Vec<T>
where
    T: FromStr,
    TArg: AsRef<OsStr>,
    TEnvKey: AsRef<OsStr>,
    TEnvVal: AsRef<OsStr>,
    <T as FromStr>::Err: StdError,
{
    execute_iter_nopanic_nopanic(cmd, args, envs).collect()
}

/// Executes command with args and environment variables, parses output line by line, returns after reading whole output
/// * On invalid command: return error
/// * On error exit code: return error
/// * On parsing failure: panic
/// * Possible errors: ProcessNotSpawned, WaitFailed, ProcessFailed
///
/// Designed for
/// ```rust
/// use shellfn::shell;
/// use std::error::Error;
///
/// #[shell]
/// fn command() -> Result<Vec<u32>, Box<Error>> {
///     "echo 1; echo 2; echo 3"
/// }
///
/// assert_eq!(vec![1, 2, 3], command().unwrap().collect::<Vec<_>>())
/// ```
pub fn execute_vec_result_panic<T, TArg, TEnvKey, TEnvVal, TError>(
    cmd: impl AsRef<OsStr>,
    args: impl IntoIterator<Item = TArg>,
    envs: impl IntoIterator<Item = (TEnvKey, TEnvVal)>,
) -> Result<Vec<T>, TError>
where
    T: FromStr,
    TArg: AsRef<OsStr>,
    TEnvKey: AsRef<OsStr>,
    TEnvVal: AsRef<OsStr>,
    <T as FromStr>::Err: StdError,
    TError: From<Error<<T as FromStr>::Err>>,
{
    let mut process = spawn(cmd, args, envs).map_err(Error::ProcessNotSpawned)?;
    let stdout = process.stdout.take().unwrap();
    let mut result = Vec::new();

    for lres in BufReader::new(stdout).lines() {
        result.push(lres.expect(PANIC_MSG).parse().expect(PANIC_MSG));
    }

    check_exit_code(process)?;
    Ok(result)
}

/// Executes command with args and environment variables, parses output line by line, returns after reading whole output
/// * On invalid command: return error
/// * On error exit code: return error
/// * On parsing failure: skip item
/// * Possible errors: ProcessNotSpawned, WaitFailed, ProcessFailed
///
/// Designed for
/// ```rust
/// use shellfn::shell;
/// use std::error::Error;
///
/// #[shell(no_panic)]
/// fn command() -> Result<Vec<u32>, Box<Error>> {
///     "echo 1; echo 2; echo 3"
/// }
///
/// assert_eq!(vec![1, 2, 3], command().unwrap().collect::<Vec<_>>())
/// ```
pub fn execute_vec_result_nopanic<T, TArg, TEnvKey, TEnvVal, TError>(
    cmd: impl AsRef<OsStr>,
    args: impl IntoIterator<Item = TArg>,
    envs: impl IntoIterator<Item = (TEnvKey, TEnvVal)>,
) -> Result<Vec<T>, TError>
where
    T: FromStr,
    TArg: AsRef<OsStr>,
    TEnvKey: AsRef<OsStr>,
    TEnvVal: AsRef<OsStr>,
    <T as FromStr>::Err: StdError,
    TError: From<Error<<T as FromStr>::Err>>,
{
    let mut process = spawn(cmd, args, envs).map_err(Error::ProcessNotSpawned)?;
    let stdout = process.stdout.take().unwrap();
    let result = BufReader::new(stdout)
        .lines()
        .filter_map(|lres| lres.ok().and_then(|line| line.parse().ok()))
        .collect::<Vec<_>>();

    check_exit_code(process)?;
    Ok(result)
}
