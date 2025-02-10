use crate::error::Error;
use crate::utils::{spawn, PANIC_MSG};
use std::error::Error as StdError;
use std::ffi::OsStr;
use std::str::FromStr;

/// Executes command with args and environment variables, parses output
/// * On invalid command: return error
/// * On error exit code: return error
/// * On parsing failure: return error
/// * Possible errors: ProcessNotSpawned, WaitFailed, ProcessFailed, NonUtf8Stdout, ParsingError
///
/// Designed for
/// ```rust
/// use shellfn::shell;
/// use std::error::Error;
///
/// #[shell]
/// fn command() -> Result<u32, Box<Error>> {
///     "echo -n 42"
/// }
///
/// assert_eq!(42, command().unwrap())
/// ```
pub fn execute_parse_result<T, TArg, TEnvKey, TEnvVal, TError>(
    cmd: impl AsRef<OsStr>,
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
    let result = process.wait_with_output().map_err(Error::WaitFailed)?;

    if !result.status.success() {
        return Err(Error::ProcessFailed(result))?;
    }

    String::from_utf8(result.stdout)
        .map_err(Error::NonUtf8Stdout)
        .map_err(Into::into)
        .and_then(|s| s.parse().map_err(Error::ParsingError).map_err(Into::into))
}

/// Executes command with args and environment variables, parses output
/// * On invalid command: panic
/// * On error exit code: panic
/// * On parsing failure: panic
/// * Possible errors: N/A
///
/// Designed for
/// ```rust
/// use shellfn::shell;
///
/// #[shell]
/// fn command() -> u32 {
///     "echo -n 42"
/// }
///
/// assert_eq!(42, command())
/// ```
pub fn execute_parse_panic<T, TArg, TEnvKey, TEnvVal>(
    cmd: impl AsRef<OsStr>,
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
        panic!("{}", PANIC_MSG);
    }

    String::from_utf8(result.stdout)
        .expect(PANIC_MSG)
        .parse()
        .expect(PANIC_MSG)
}
