use crate::error::Error;
use crate::utils::{spawn, PANIC_MSG};
use itertools::Either;
use std::error::Error as StdError;
use std::ffi::OsStr;
use std::io::{BufRead, BufReader};
use std::str::FromStr;

/// Executes command with args and environment variables, parses output line by line
/// * On invalid command: return error
/// * On error exit code: break iterator
/// * On parsing failure: yield error item
/// * Possible errors: ProcessNotSpawned, StdoutUnreadable (item error), ParsingError (item error)
///
/// Designed for
/// ```rust
/// use shellfn::shell;
/// use std::error::Error;
///
/// #[shell]
/// fn command() -> Result<impl Iterator<Item = Result<u32, Box<Error + 'static>>>, Box<Error>> {
///     "echo 1; echo 2; echo 3"
/// }
///
/// assert_eq!(vec![1, 2, 3], command().unwrap().map(Result::unwrap).collect::<Vec<_>>())
/// ```
pub fn execute_iter_result_result<T, TArg, TEnvKey, TEnvVal, TOuterError, TInnerError>(
    cmd: impl AsRef<OsStr>,
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
    let stdout = process.stdout.take().unwrap();

    Ok(BufReader::new(stdout).lines().map(|lres|
        lres.map_err(Error::StdoutUnreadable)
            .map_err(Into::into)
            .and_then(|line|
                line.parse()
                    .map_err(Error::ParsingError)
                    .map_err(Into::into)
            )
    ))
}

/// Executes command with args and environment variables, parses output line by line
/// * On invalid command: panic
/// * On error exit code: break iterator
/// * On parsing failure: panic
/// * Possible errors: N/A
///
/// Designed for
/// ```rust
/// use shellfn::shell;
/// use std::error::Error;
///
/// #[shell]
/// fn command() -> impl Iterator<Item = u32> {
///     "echo 1; echo 2; echo 3"
/// }
///
/// assert_eq!(vec![1, 2, 3], command().collect::<Vec<_>>())
/// ```
pub fn execute_iter_panic_panic<T, TArg, TEnvKey, TEnvVal>(
    cmd: impl AsRef<OsStr>,
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
    let stdout = process.stdout.take().unwrap();

    BufReader::new(stdout)
        .lines()
        .map(|lres| lres.expect(PANIC_MSG).parse().expect(PANIC_MSG))
        .chain([()].into_iter().flat_map(move |_| {
            if !process.wait().unwrap().success() {
                panic!("{}", PANIC_MSG)
            }
            std::iter::empty()
        }))
}

/// Executes command with args and environment variables, parses output line by line
/// * On invalid command: panic
/// * On error exit code: break iterator
/// * On parsing failure: yield error item
/// * Possible errors: StdoutUnreadable (item error), ParsingError (item error)
///
/// Designed for
/// ```rust
/// use shellfn::shell;
/// use std::error::Error;
///
/// #[shell]
/// fn command() -> impl Iterator<Item = Result<u32, Box<Error + 'static>>> {
///     "echo 1; echo 2; echo 3"
/// }
///
/// assert_eq!(vec![1, 2, 3], command().map(Result::unwrap).collect::<Vec<_>>())
/// ```
pub fn execute_iter_panic_result<T, TArg, TEnvKey, TEnvVal, TError>(
    cmd: impl AsRef<OsStr>,
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
    let stdout = process.stdout.take().unwrap();

    BufReader::new(stdout)
        .lines()
        .map(|lres|
            lres.map_err(Error::StdoutUnreadable)
                .map_err(Into::into)
                .and_then(|line|
                    line.parse()
                        .map_err(Error::ParsingError)
                        .map_err(Into::into)
                )
        )
        .chain([()].into_iter().flat_map(move |_| {
            if !process.wait().unwrap().success() {
                panic!("{}", PANIC_MSG)
            }
            std::iter::empty()
        }))
}

/// Executes command with args and environment variables, parses output line by line
/// * On invalid command: return empty iterator
/// * On error exit code: break iterator
/// * On parsing failure: yield error item
/// * Possible errors: StdoutUnreadable (item error), ParsingError (item error)
///
/// Designed for
/// ```rust
/// use shellfn::shell;
/// use std::error::Error;
///
/// #[shell(no_panic)]
/// fn command() -> impl Iterator<Item = Result<u32, Box<Error + 'static>>> {
///     "echo 1; echo 2; echo 3"
/// }
///
/// assert_eq!(vec![1, 2, 3], command().map(Result::unwrap).collect::<Vec<_>>())
/// ```
pub fn execute_iter_nopanic_result<T, TArg, TEnvKey, TEnvVal, TError>(
    cmd: impl AsRef<OsStr>,
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
        .map(|mut process|
            BufReader::new(process.stdout.take().unwrap())
                .lines()
                .map(|lres|
                    lres.map_err(Error::StdoutUnreadable)
                        .map_err(Into::into)
                        .and_then(|line|
                            line.parse()
                                .map_err(Error::ParsingError)
                                .map_err(Into::into)
                        )
                )
        )
        .map_or_else(
            || Either::Right(std::iter::empty()),
            |iter| Either::Left(iter),
        )
}

/// Executes command with args and environment variables, parses output line by line
/// * On invalid command: return empty iterator
/// * On error exit code: break iterator
/// * On parsing failure: skip item
/// * Possible errors: N/A
///
/// Designed for
/// ```rust
/// use shellfn::shell;
/// use std::error::Error;
///
/// #[shell(no_panic)]
/// fn command() -> impl Iterator<Item = u32> {
///     "echo 1; echo 2; echo 3"
/// }
///
/// assert_eq!(vec![1, 2, 3], command().collect::<Vec<_>>())
/// ```
pub fn execute_iter_nopanic_nopanic<T, TArg, TEnvKey, TEnvVal>(
    cmd: impl AsRef<OsStr>,
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

/// Executes command with args and environment variables, parses output line by line
/// * On invalid command: return error
/// * On error exit code: break iterator
/// * On parsing failure: panic
/// * Possible errors: ProcessNotSpawned
///
/// Designed for
/// ```rust
/// use shellfn::shell;
/// use std::error::Error;
///
/// #[shell]
/// fn command() -> Result<impl Iterator<Item = u32>, Box<Error>> {
///     "echo 1; echo 2; echo 3"
/// }
///
/// assert_eq!(vec![1, 2, 3], command().unwrap().collect::<Vec<_>>())
/// ```
pub fn execute_iter_result_panic<T, TArg, TEnvKey, TEnvVal, TError>(
    cmd: impl AsRef<OsStr>,
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
    let stdout = process.stdout.take().unwrap();

    Ok(BufReader::new(stdout)
        .lines()
        .map(|lres| lres.expect(PANIC_MSG).parse().expect(PANIC_MSG)))
}

/// Executes command with args and environment variables, parses output line by line
/// * On invalid command: return error
/// * On error exit code: break iterator
/// * On parsing failure: skip item
/// * Possible errors: ProcessNotSpawned
///
/// Designed for
/// ```rust
/// use shellfn::shell;
/// use std::error::Error;
///
/// #[shell(no_panic)]
/// fn command() -> Result<impl Iterator<Item = u32>, Box<Error>> {
///     "echo 1; echo 2; echo 3"
/// }
///
/// assert_eq!(vec![1, 2, 3], command().unwrap().collect::<Vec<_>>())
/// ```
pub fn execute_iter_result_nopanic<T, TArg, TEnvKey, TEnvVal, TError>(
    cmd: impl AsRef<OsStr>,
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
    let stdout = process.stdout.take().unwrap();

    Ok(BufReader::new(stdout)
        .lines()
        .filter_map(|lres| lres.ok().and_then(|item| item.parse().ok())))
}
