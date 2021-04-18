pub mod core;
pub mod func;
pub mod info;
pub mod preview;
pub mod preview_var;
pub mod preview_var_win;
pub mod repo_add;
pub mod repo_browse;
pub mod shell;

use crate::config::Command::{Fn, Info, Preview, PreviewVar, PreviewVarWin, Repo, Widget};
use crate::config::{RepoCommand, CONFIG};
use crate::handler;
use anyhow::Context;
use anyhow::Result;

pub fn handle() -> Result<()> {
    match CONFIG.cmd() {
        None => handler::core::main(),

        Some(c) => match c {
            Preview { line } => handler::preview::main(line),

            PreviewVarWin => handler::preview_var_win::main(),

            PreviewVar {
                selection,
                query,
                variable,
            } => handler::preview_var::main(selection, query, variable),

            Widget { shell } => handler::shell::main(shell).context("Failed to print shell widget code"),

            Fn { func, args } => handler::func::main(func, args.to_vec())
                .with_context(|| format!("Failed to execute function `{:#?}`", func)),

            Info { info } => {
                handler::info::main(info).with_context(|| format!("Failed to fetch info `{:#?}`", info))
            }

            Repo { cmd } => match cmd {
                RepoCommand::Add { uri } => {
                    handler::repo_add::main(uri.clone())
                        .with_context(|| format!("Failed to import cheatsheets from `{}`", uri))?;
                    handler::core::main()
                }
                RepoCommand::Browse => {
                    let repo =
                        handler::repo_browse::main().context("Failed to browse featured cheatsheets")?;
                    handler::repo_add::main(repo.clone())
                        .with_context(|| format!("Failed to import cheatsheets from `{}`", repo))?;
                    handler::core::main()
                }
            },
        },
    }
}
