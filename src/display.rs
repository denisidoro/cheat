use crate::terminal;

use regex::Regex;
use std::cmp::max;
use termion::color;

static COMMENT_COLOR: color::LightCyan = color::LightCyan;
static TAG_COLOR: color::Blue = color::Blue;
static SNIPPET_COLOR: color::White = color::White;

static NEWLINE_ESCAPE_CHAR: char = '\x15';
pub static LINE_SEPARATOR: &str = " \x15 ";
pub static DELIMITER: &str = r"  ⠀";

lazy_static! {
    pub static ref WIDTHS: (usize, usize) = get_widths();
}

fn get_widths() -> (usize, usize) {
    let width = terminal::width();
    let tag_width = max(4, width * 20 / 100);
    let comment_width = max(4, width * 40 / 100);
    (usize::from(tag_width), usize::from(comment_width))
}

pub fn variable_prompt(varname: &str) -> String {
    format!("{}: ", varname)
}

fn fix_newlines(txt: &str) -> String {
    if txt.contains(NEWLINE_ESCAPE_CHAR) {
        let re = Regex::new(r"\\\s+").unwrap();
        re.replace_all(txt.replace(LINE_SEPARATOR, "  ").as_str(), "")
            .to_string()
    } else {
        txt.to_string()
    }
}

pub fn preview(comment: &str, tags: &str, snippet: &str) {
    println!(
        "{comment_color}{comment} {tag_color}{tags} \n{snippet_color}{snippet}",
        comment = format!("# {}", comment),
        tags = format!("[{}]", tags),
        snippet = fix_newlines(snippet),
        comment_color = color::Fg(COMMENT_COLOR),
        tag_color = color::Fg(TAG_COLOR),
        snippet_color = color::Fg(SNIPPET_COLOR),
    );
}

fn limit_str(text: &str, length: usize) -> String {
    if text.len() > length {
        format!("{}…", &text[..length - 1])
    } else {
        format!("{:width$}", text, width = length)
    }
}

pub fn format_line(
    tags: &str,
    comment: &str,
    snippet: &str,
    tag_width: usize,
    comment_width: usize,
) -> String {
    format!(
       "{tag_color}{tags_short}{delimiter}{comment_color}{comment_short}{delimiter}{snippet_color}{snippet_short}{delimiter}{tags}{delimiter}{comment}{delimiter}{snippet}{delimiter}\n",
       tags_short = limit_str(tags, tag_width),
       comment_short = limit_str(comment, comment_width),
       snippet_short = fix_newlines(snippet),
       comment_color = color::Fg(COMMENT_COLOR),
       tag_color = color::Fg(TAG_COLOR),
       snippet_color = color::Fg(SNIPPET_COLOR),
       tags = tags,
       comment = comment,
       delimiter = DELIMITER,
       snippet = &snippet)
}
