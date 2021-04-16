use crate::env_var;
use crate::finder;
use crate::structures::item::Item;
use crate::terminal;
use crate::terminal::style::{style, Color};
use crate::ui;
use crate::writer;
use anyhow::Result;
use std::cmp::max;
use std::collections::HashSet;
use std::iter;
use std::process;

pub fn main(selection: &str, query: &str, variable: &str) -> Result<()> {
    let snippet = env_var::must_get(env_var::PREVIEW_INITIAL_SNIPPET);
    let tags = env_var::must_get(env_var::PREVIEW_TAGS);
    let comment = env_var::must_get(env_var::PREVIEW_COMMENT);
    let column = env_var::parse(env_var::PREVIEW_COLUMN);
    let delimiter = env_var::get(env_var::PREVIEW_DELIMITER).ok();
    let map = env_var::get(env_var::PREVIEW_MAP).ok();

    let active_color = *ui::TAG_COLOR;
    let inactive_color = *ui::COMMENT_COLOR;

    let mut colored_snippet = String::from(&snippet);
    let mut visited_vars: HashSet<&str> = HashSet::new();

    let mut variables = String::from("");

    println!(
        "{comment} {tags}",
        comment = style(comment).with(*ui::COMMENT_COLOR),
        tags = style(format!("[{}]", tags)).with(*ui::TAG_COLOR),
    );

    let bracketed_current_variable = format!("<{}>", variable);

    let bracketed_variables: Vec<&str> = {
        if snippet.contains(&bracketed_current_variable) {
            writer::VAR_REGEX
                .find_iter(&snippet)
                .map(|m| m.as_str())
                .collect()
        } else {
            iter::once(&bracketed_current_variable)
                .map(|s| s.as_str())
                .collect()
        }
    };

    for bracketed_variable_name in bracketed_variables {
        let variable_name = &bracketed_variable_name[1..bracketed_variable_name.len() - 1];

        if visited_vars.contains(variable_name) {
            continue;
        } else {
            visited_vars.insert(variable_name);
        }

        let is_current = variable_name == variable;
        let variable_color = if is_current { active_color } else { inactive_color };
        let env_variable_name = env_var::escape(variable_name);

        let value = if is_current {
            let v = selection.trim_matches('\'');
            if v.is_empty() { query.trim_matches('\'') } else { v }.to_string()
        } else if let Ok(v) = env_var::get(&env_variable_name) {
            v
        } else {
            "".to_string()
        };

        let replacement = format!(
            "{variable}",
            variable = style(bracketed_variable_name).with(variable_color),
        );

        colored_snippet = colored_snippet.replace(bracketed_variable_name, &replacement);

        variables = format!(
            "{variables}\n{variable} = {value}",
            variables = variables,
            variable = style(variable_name).with(variable_color),
            value = finder::process(value, column, delimiter.as_deref(), map.clone())
                .expect("Unable to process value"),
        );
    }

    println!("{snippet}", snippet = writer::fix_newlines(&colored_snippet));
    println!("{variables}", variables = variables);

    process::exit(0)
}
