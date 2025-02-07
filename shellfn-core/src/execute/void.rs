use crate::error::{Error, NeverError};
use crate::utils::{spawn, PANIC_MSG};
use std::ffi::OsStr;
use std::process::{Child, Output};

/// Executes command with args and environment variables, ignores output
/// * On invalid command: do nothing
/// * On error exit code: do nothing
/// * On parsing failure: N/A
/// * Possible errors: N/A
///
/// Designed for
/// ```rust
/// use shellfn::shell;
///
/// #[shell(no_panic)]
/// fn command() {
///     "echo Hello, world"
/// }
///
/// command()
/// ```
pub fn execute_void_nopanic<TArg, TEnvKey, TEnvVal>(
    cmd: impl AsRef<OsStr>,
    args: impl IntoIterator<Item = TArg>,
    envs: impl IntoIterator<Item = (TEnvKey, TEnvVal)>,
) where
    TArg: AsRef<OsStr>,
    TEnvKey: AsRef<OsStr>,
    TEnvVal: AsRef<OsStr>,
{
    let _ = spawn(cmd, args, envs).and_then(Child::wait_with_output);
}

/// Executes command with args and environment variables, ignores output
/// * On invalid command: panic
/// * On error exit code: panic
/// * On parsing failure: N/A
/// * Possible errors: N/A
///
/// Designed for
/// ```rust
/// use shellfn::shell;
///
/// #[shell]
/// fn command() {
///     "echo Hello, world"
/// }
///
/// command()
/// ```
pub fn execute_void_panic<TArg, TEnvKey, TEnvVal>(
    cmd: impl AsRef<OsStr>,
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
        panic!("{}", PANIC_MSG)
    }
}

/// Executes command with args and environment variables, ignores output
/// * On invalid command: return error
/// * On error exit code: return error
/// * On parsing failure: N/A
/// * Possible errors: ProcessNotSpawned, WaitFailed, ProcessFailed (stdout and stderr always empty)
///
/// Designed for
/// ```rust
/// use shellfn::shell;
///
/// #[shell]
/// fn command() -> Result<(), Box<Error>> {
///     sleep 5
/// }
///
/// command()
/// ```
pub fn execute_void_result<TArg, TEnvKey, TEnvVal, TError>(
    cmd: impl AsRef<OsStr>,
    args: impl IntoIterator<Item = TArg>,
    envs: impl IntoIterator<Item = (TEnvKey, TEnvVal)>,
) -> Result<(), TError>
where
    TArg: AsRef<OsStr>,
    TEnvKey: AsRef<OsStr>,
    TEnvVal: AsRef<OsStr>,
    TError: From<Error<NeverError>>,
{
    let mut process = spawn(cmd, args, envs).map_err(Error::ProcessNotSpawned)?;
    let status = process.wait().map_err(Error::WaitFailed)?;

    if !status.success() {
        return Err(Error::ProcessFailed(Output {
            status,
            stdout: Vec::new(),
            stderr: Vec::new(),
        }))?;
    }

    Ok(())
}
