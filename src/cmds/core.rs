use crate::cheat;
use crate::cmds;
use crate::display;
use crate::fzf;
use crate::option::Config;

use crate::cheat::SuggestionType;
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

pub enum Variant {
    Core,
    Filter(String),
    Query(String),
}

fn gen_core_fzf_opts(variant: Variant, config: &Config) -> fzf::Opts {
    let mut opts = fzf::Opts {
        preview: !config.no_preview,
        autoselect: !config.no_autoselect,
        overrides: config.fzf_overrides.as_ref(),
        suggestion_type: SuggestionType::SnippetSelection,
        ..Default::default()
    };

    match variant {
        Variant::Core => (),
        Variant::Filter(f) => opts.filter = Some(f),
        Variant::Query(q) => opts.query = Some(q),
    }

    opts
}

fn extract_from_selections(raw_snippet: &str, contains_key: bool) -> (&str, &str, &str) {
    let mut lines = raw_snippet.split('\n');
    let key = if contains_key {
        lines.next().unwrap()
    } else {
        "enter"
    };

    let mut parts = lines.next().unwrap().split(display::DELIMITER);
    parts.next();
    parts.next();
    parts.next();

    let tags = parts.next().unwrap_or("");
    parts.next();

    let snippet = parts.next().unwrap_or("");
    (key, tags, snippet)
}

fn prompt_with_suggestions(
    varname: &str,
    config: &Config,
    suggestion: &cheat::Suggestion,
    values: &HashMap<String, String>,
) -> String {
    let mut vars_cmd = String::from("");
    for (key, value) in values.iter() {
        vars_cmd.push_str(format!("{}=\"{}\"; ", key, value).as_str());
    }
    let (suggestion_command, suggestion_options) = &suggestion;
    let command = format!("{} {}", vars_cmd, suggestion_command);

    let child = Command::new("bash")
        .stdout(Stdio::piped())
        .arg("-c")
        .arg(command)
        .spawn()
        .unwrap();

    let suggestions = String::from_utf8(child.wait_with_output().unwrap().stdout).unwrap();

    let mut opts = fzf::Opts {
        preview: false,
        autoselect: !config.no_autoselect,
        overrides: config.fzf_overrides_var.as_ref(),
        prompt: Some(display::variable_prompt(varname)),
        ..Default::default()
    };

    let mut column: Option<u8> = None;
    let mut delimiter = r"\s\s+";

    if let Some(o) = &suggestion_options {
        opts.suggestion_type = o.suggestion_type;
        opts.header_lines = o.header_lines;
        column = o.column;
        if let Some(d) = o.delimiter.as_ref() {
            delimiter = d.as_str();
        }
    };

    let (output, _) = fzf::call(opts, |stdin| {
        stdin.write_all(suggestions.as_bytes()).unwrap();
        None
    });

    if let Some(c) = column {
        let re = regex::Regex::new(delimiter).unwrap();
        let mut parts = re.split(output.as_str());
        for _ in 0..(c - 1) {
            parts.next().unwrap();
        }
        parts.next().unwrap().to_string()
    } else {
        output
    }
}

fn prompt_without_suggestions(variable_name: &str) -> String {
    let opts = fzf::Opts {
        preview: false,
        autoselect: false,
        prompt: Some(display::variable_prompt(variable_name)),
        suggestion_type: SuggestionType::Disabled,
        ..Default::default()
    };

    let (output, _) = fzf::call(opts, |_stdin| None);

    output
}

fn gen_replacement(value: &str) -> String {
    if value.contains(' ') {
        format!("\"{}\"", value)
    } else {
        value.to_string()
    }
}

fn replace_variables_from_snippet(
    snippet: &str,
    tags: &str,
    variables: HashMap<String, cheat::Suggestion>,
    config: &Config,
) -> String {
    let mut interpolated_snippet = String::from(snippet);
    let mut values: HashMap<String, String> = HashMap::new();

    let re = Regex::new(r"<(\w[\w\d\-_]*)>").unwrap();
    for captures in re.captures_iter(snippet) {
        let bracketed_variable_name = &captures[0];
        let variable_name = &bracketed_variable_name[1..bracketed_variable_name.len() - 1];

        if values.get(variable_name).is_none() {
            let key = format!("{};{}", tags, variable_name);

            let value = match variables.get(&key[..]) {
                Some(suggestion) => {
                    prompt_with_suggestions(variable_name, &config, suggestion, &values)
                }
                None => prompt_without_suggestions(variable_name),
            };

            values.insert(variable_name.to_string(), value.clone());

            interpolated_snippet = interpolated_snippet.replace(
                bracketed_variable_name,
                gen_replacement(&value[..]).as_str(),
            );
        }
    }

    interpolated_snippet
}

fn with_new_lines(txt: String) -> String {
    txt.replace(display::LINE_SEPARATOR, "\n")
}

pub fn main(variant: Variant, config: Config, contains_key: bool) -> Result<(), Box<dyn Error>> {
    let _ = display::WIDTHS;

    let (raw_selection, variables) = fzf::call(gen_core_fzf_opts(variant, &config), |stdin| {
        Some(cheat::read_all(&config, stdin))
    });

    let (key, tags, snippet) = extract_from_selections(&raw_selection[..], contains_key);
    let interpolated_snippet = with_new_lines(replace_variables_from_snippet(
        snippet,
        tags,
        variables.unwrap(),
        &config,
    ));

    if key == "ctrl-y" {
        cmds::aux::abort("copying snippets to the clipboard", 201)?
    } else if config.print {
        println!("{}", interpolated_snippet);
    } else if let Some(s) = config.save {
        fs::write(s, interpolated_snippet)?;
    } else {
        Command::new("bash")
            .arg("-c")
            .arg(&interpolated_snippet[..])
            .spawn()?;
    }

    Ok(())
}
