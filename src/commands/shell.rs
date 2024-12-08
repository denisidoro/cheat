use std::fmt;
use std::fmt::Display;

use clap::Args;

use crate::common::shell::Shell;
use crate::prelude::*;

impl Display for Shell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Bash => "bash",
            Self::Zsh => "zsh",
            Self::Fish => "fish",
            Self::Elvish => "elvish",
            Self::Nushell => "nushell",
            Self::PowerShell => "powershell"
        };

        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, Args)]
pub struct Input {
    #[clap(ignore_case = true, default_value_t = Shell::Bash)]
    pub shell: Shell,
}

impl Runnable for Input {
    fn run(&self) -> Result<()> {
        let shell = &self.shell;

        let content = match shell {
            Shell::Bash => include_str!("../../shell/navi.plugin.bash"),
            Shell::Zsh => include_str!("../../shell/navi.plugin.zsh"),
            Shell::Fish => include_str!("../../shell/navi.plugin.fish"),
            Shell::Elvish => include_str!("../../shell/navi.plugin.elv"),
            Shell::Nushell => include_str!("../../shell/navi.plugin.nu"),
            Shell::PowerShell => include_str!("../../shell/navi.plugin.ps1"),
        };

        println!("{content}");

        Ok(())
    }
}
