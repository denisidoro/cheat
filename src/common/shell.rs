use crate::prelude::*;
use clap::ValueEnum;
use std::process::Command;
use thiserror::Error;

pub const EOF: &str = "NAVIEOF";

#[derive(Debug, Clone, ValueEnum)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    Elvish,
    Nushell,
    PowerShell
}

#[derive(Error, Debug)]
#[error("Failed to spawn child process `bash` to execute `{command}`")]
pub struct ShellSpawnError {
    command: String,
    #[source]
    source: anyhow::Error,
}

impl ShellSpawnError {
    pub fn new<SourceError>(command: impl Into<String>, source: SourceError) -> Self
    where
        SourceError: std::error::Error + Sync + Send + 'static,
    {
        ShellSpawnError {
            command: command.into(),
            source: source.into(),
        }
    }
}

pub fn out() -> Command {
    let words_str = CONFIG.shell();
    let mut words_vec = shellwords::split(&words_str).expect("empty shell command");
    let mut words = words_vec.iter_mut();
    let first_cmd = words.next().expect("absent shell binary");
    let mut cmd = Command::new(first_cmd);
    cmd.args(words);
    let dash_c = if words_str.contains("cmd.exe") { "/c" } else { "-c" };
    cmd.arg(dash_c);
    cmd
}
