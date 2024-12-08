use crate::common::clipboard;
use crate::common::fs;
use crate::common::shell;
use crate::common::shell::ShellSpawnError;
use crate::config::Action;
use crate::deser;
use crate::env_var;
use crate::finder::structures::{Opts as FinderOpts, SuggestionType};
use crate::prelude::*;
use crate::structures::cheat::{Suggestion, VariableMap};
use crate::structures::item::Item;
use shell::EOF;
use std::process::Stdio;

fn prompt_finder(
    variable_name: &str,
    suggestion: Option<&Suggestion>,
    variable_count: usize,
) -> Result<String> {
    env_var::remove(env_var::PREVIEW_COLUMN);
    env_var::remove(env_var::PREVIEW_DELIMITER);
    env_var::remove(env_var::PREVIEW_MAP);

    let mut extra_preview: Option<String> = None;

    let (suggestions, initial_opts) = if let Some(s) = suggestion {
        let (suggestion_command, suggestion_opts) = s;

        if let Some(sopts) = suggestion_opts {
            if let Some(c) = &sopts.column {
                env_var::set(env_var::PREVIEW_COLUMN, c.to_string());
            }
            if let Some(d) = &sopts.delimiter {
                env_var::set(env_var::PREVIEW_DELIMITER, d);
            }
            if let Some(m) = &sopts.map {
                env_var::set(env_var::PREVIEW_MAP, m);
            }
            if let Some(p) = &sopts.preview {
                extra_preview = Some(p.into());
            }
        }

        let mut cmd = shell::out();
        cmd.stdout(Stdio::piped()).arg(suggestion_command);
        debug!(cmd = ?cmd);
        let child = cmd
            .spawn()
            .map_err(|e| ShellSpawnError::new(suggestion_command, e))?;

        let text = String::from_utf8(
            child
                .wait_with_output()
                .context("Failed to wait and collect output from bash")?
                .stdout,
        )
        .context("Suggestions are invalid utf8")?;

        (text, suggestion_opts)
    } else {
        ('\n'.to_string(), &None)
    };

    let exe = fs::exe_string();

    let preview = if CONFIG.shell().contains("powershell"){
        format!(
            r#"{exe} preview-var {{+}} "{{q}}" "{name}"; {extra}"#,
            exe = exe,
            name = variable_name,
            extra = extra_preview
                .clone()
                .map(|e| format!(" echo; {e}"))
                .unwrap_or_default(),
        )
    } else if CONFIG.shell().contains("cmd.exe") {
        format!(
            r#"(@echo.{{+}}{eof}{{q}}{eof}{name}{eof}{extra}) | {exe} preview-var-stdin"#,
            exe = exe,
            name = variable_name,
            extra = extra_preview.clone().unwrap_or_default(),
            eof = EOF,
        )
    } else if CONFIG.shell().contains("fish") {
        format!(
            r#"{exe} preview-var "{{+}}" "{{q}}" "{name}"; {extra}"#,
            exe = exe,
            name = variable_name,
            extra = extra_preview
                .clone()
                .map(|e| format!(" echo; {e}"))
                .unwrap_or_default(),
        )
    } else {
        format!(
            r#"{exe} preview-var "$(cat <<{eof}
{{+}}
{eof}
)" "$(cat <<{eof}
{{q}}
{eof}
)" "{name}"; {extra}"#,
            exe = exe,
            name = variable_name,
            extra = extra_preview
                .clone()
                .map(|e| format!(" echo; {e}"))
                .unwrap_or_default(),
            eof = EOF,
        )
    };

    let mut opts = FinderOpts {
        preview: Some(preview),
        show_all_columns: true,
        ..initial_opts.clone().unwrap_or_else(FinderOpts::var_default)
    };

    opts.query = env_var::get(format!("{variable_name}__query")).ok();

    if let Ok(f) = env_var::get(format!("{variable_name}__best")) {
        opts.filter = Some(f);
        opts.suggestion_type = SuggestionType::SingleSelection;
    }

    if opts.preview_window.is_none() {
        opts.preview_window = Some(if extra_preview.is_none() {
            format!("up:{}", variable_count + 3)
        } else {
            "right:50%".to_string()
        });
    }

    if suggestion.is_none() {
        opts.suggestion_type = SuggestionType::Disabled;
    };

    let (output, _) = CONFIG
        .finder()
        .call(opts, |stdin| {
            stdin
                .write_all(suggestions.as_bytes())
                .context("Could not write to finder's stdin")?;
            Ok(())
        })
        .context("finder was unable to prompt with suggestions")?;

    Ok(output)
}

fn unique_result_count(results: &[&str]) -> usize {
    let mut vars = results.to_owned();
    vars.sort_unstable();
    vars.dedup();
    vars.len()
}

fn replace_variables_from_snippet(snippet: &str, tags: &str, variables: VariableMap) -> Result<String> {
    let mut interpolated_snippet = String::from(snippet);

    if CONFIG.prevent_interpolation() {
        return Ok(interpolated_snippet);
    }

    let variables_found: Vec<&str> = deser::VAR_REGEX.find_iter(snippet).map(|m| m.as_str()).collect();
    let variable_count = unique_result_count(&variables_found);

    for bracketed_variable_name in variables_found {
        let variable_name = &bracketed_variable_name[1..bracketed_variable_name.len() - 1];

        let env_variable_name = env_var::escape(variable_name);
        let env_value = env_var::get(&env_variable_name);

        let value = if let Ok(e) = env_value {
            e
        } else if let Some(suggestion) = variables.get_suggestion(tags, variable_name) {
            let mut new_suggestion = suggestion.clone();
            new_suggestion.0 = replace_variables_from_snippet(&new_suggestion.0, tags, variables.clone())?;
            prompt_finder(variable_name, Some(&new_suggestion), variable_count)?
        } else {
            prompt_finder(variable_name, None, variable_count)?
        };

        env_var::set(env_variable_name, &value);

        interpolated_snippet = if value.as_str() == "\n" {
            interpolated_snippet.replacen(bracketed_variable_name, "", 1)
        } else {
            interpolated_snippet.replacen(bracketed_variable_name, value.as_str(), 1)
        };
    }

    Ok(interpolated_snippet)
}

pub fn with_absolute_path(snippet: String) -> String {
    if let Some(s) = snippet.strip_prefix("navi ") {
        return format!("{} {}", fs::exe_string(), s);
    }
    snippet
}

pub fn act(
    extractions: Result<(&str, Item)>,
    files: Vec<String>,
    variables: Option<VariableMap>,
) -> Result<()> {
    let (
        key,
        Item {
            tags,
            comment,
            snippet,
            file_index,
            ..
        },
    ) = extractions.unwrap();

    if key == "ctrl-o" {
        edit::edit_file(Path::new(&files[file_index.expect("No files found")]))
            .expect("Could not open file in external editor");
        return Ok(());
    }

    env_var::set(env_var::PREVIEW_INITIAL_SNIPPET, &snippet);
    env_var::set(env_var::PREVIEW_TAGS, &tags);
    env_var::set(env_var::PREVIEW_COMMENT, comment);

    let interpolated_snippet = {
        let mut s = replace_variables_from_snippet(
            &snippet,
            &tags,
            variables.expect("No variables received from finder"),
        )
        .context("Failed to replace variables from snippet")?;
        s = with_absolute_path(s);
        s = deser::with_new_lines(s);
        s
    };

    match CONFIG.action() {
        Action::Print => {
            println!("{interpolated_snippet}");
        }
        Action::Execute => match key {
            "ctrl-y" => {
                clipboard::copy(interpolated_snippet)?;
            }
            _ => {
                let mut cmd = shell::out();
                cmd.arg(&interpolated_snippet[..]);
                debug!(cmd = ?cmd);
                cmd.spawn()
                    .map_err(|e| ShellSpawnError::new(&interpolated_snippet[..], e))?
                    .wait()
                    .context("bash was not running")?;
            }
        },
    };

    Ok(())
}
