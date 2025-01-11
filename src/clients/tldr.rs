use crate::config::CONFIG;
use crate::prelude::*;
use std::process::{Command, Stdio};

lazy_static! {
    pub static ref VAR_TLDR_REGEX: Regex = Regex::new(r"\{\{(.*?)\}\}").expect("Invalid regex");
    pub static ref NON_VAR_CHARS_REGEX: Regex = Regex::new(r"[^\da-zA-Z_]").expect("Invalid regex");
}

static VERSION_DISCLAIMER: &str =
    "tldr-c-client (the default one in Homebrew) doesn't support markdown files, so navi can't use it.
The recommended client is tealdeer(https://github.com/dbrgn/tealdeer).";

fn convert_tldr_vars(line: &str) -> String {
    let caps = VAR_TLDR_REGEX.find_iter(line);
    let mut new_line: String = line.to_string();
    for cap in caps {
        let braced_var = cap.as_str();
        let var = &braced_var[2..braced_var.len() - 2];
        let mut new_var = NON_VAR_CHARS_REGEX.replace_all(var, "_").to_string();
        if let Some(c) = new_var.chars().next() {
            if c.to_string().parse::<u8>().is_ok() {
                new_var = format!("example_{new_var}");
            }
        }
        let bracketed_var = format!("<{new_var}>");
        new_line = new_line.replace(braced_var, &bracketed_var);
    }
    new_line
}

fn convert_tldr(line: &str) -> String {
    let line = line.trim();
    if line.starts_with('-') {
        format!("{}{}", "# ", &line[2..line.len() - 1])
    } else if line.starts_with('`') {
        convert_tldr_vars(&line[1..line.len() - 1])
    } else if line.starts_with('%') {
        line.to_string()
    } else {
        "".to_string()
    }
}

fn markdown_lines(query: &str, markdown: &str) -> Vec<String> {
    format!(
        "% {query}, tldr
 {markdown}"
    )
    .lines()
    .map(convert_tldr)
    .collect()
}

pub fn call(query: &str) -> Result<Vec<String>> {
    let tealdeer = CONFIG.tealdeer();
    let output_flag = if tealdeer { "--raw" } else { "--markdown" };
    let args = [query, output_flag];

    let child = Command::new("tldr")
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    let child = match child {
        Ok(x) => x,
        Err(_) => {
            let msg = format!(
                "navi was unable to call tldr.
Make sure tldr is correctly installed.

Note:
{VERSION_DISCLAIMER}
"
            );
            return Err(anyhow!(msg));
        }
    };

    let out = child.wait_with_output().context("Failed to wait for tldr")?;

    if let Some(0) = out.status.code() {
        let stdout = out.stdout;

        let markdown = String::from_utf8(stdout).context("Output is invalid utf8")?;
        let lines = markdown_lines(query, &markdown);
        Ok(lines)
    } else {
        let msg = format!(
            "Failed to call:
tldr {}

Output:
{}

Error:
{}

Note:
The client.tealdeer config option can be set to enable tealdeer support.
If you want to use another client, please make sure it supports the --markdown flag.
If you are already using a supported version you can ignore this message.
{}
",
            args.join(" "),
            String::from_utf8(out.stdout).unwrap_or_else(|_e| "Unable to get output message".to_string()),
            String::from_utf8(out.stderr).unwrap_or_else(|_e| "Unable to get error message".to_string()),
            VERSION_DISCLAIMER,
        );
        Err(anyhow!(msg))
    }
}
