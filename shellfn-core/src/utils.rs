use std::ffi::OsStr;
use std::io;
use std::process::{Child, Command, Stdio};

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
