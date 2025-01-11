use crate::prelude::*;
use std::process::Command;

fn map_line(line: &str) -> String {
    line.trim().trim_end_matches(':').to_string()
}

fn as_lines(query: &str, markdown: &str) -> Vec<String> {
    format!(
        "% {query}, cheat.sh
{markdown}"
    )
    .lines()
    .map(map_line)
    .collect()
}

pub fn call(query: &str) -> Result<Vec<String>> {
    let args = ["-qO-", &format!("cheat.sh/{query}")];

    let child = Command::new("wget")
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn();

    let child = match child {
        Ok(x) => x,
        Err(_) => {
            let msg = "navi was unable to call wget.
Make sure wget is correctly installed.";
            return Err(anyhow!(msg));
        }
    };

    let out = child.wait_with_output().context("Failed to wait for wget")?;

    if let Some(0) = out.status.code() {
        let stdout = out.stdout;
        let plain_bytes = strip_ansi_escapes::strip(stdout);

        let markdown = String::from_utf8(plain_bytes).context("Output is invalid utf8")?;
        if markdown.starts_with("Unknown topic.") {
            let msg = format!(
                "`{}` not found in cheatsh.
Output:
{}
",
                &query, markdown,
            );
            return Err(anyhow!(msg));
        }

        let lines = as_lines(query, &markdown);
        Ok(lines)
    } else {
        let msg = format!(
            "Failed to call:
wget {}

Output:
{}

Error:
{}
",
            args.join(" "),
            String::from_utf8(out.stdout).unwrap_or_else(|_e| "Unable to get output message".to_string()),
            String::from_utf8(out.stderr).unwrap_or_else(|_e| "Unable to get error message".to_string())
        );
        Err(anyhow!(msg))
    }
}
