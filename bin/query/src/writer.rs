use core::fmt::Display;
use owo_colors::OwoColorize;
use serde_json::json;
use std::io::{self, Write};

pub fn write_err<T: Display, W: Write>(item: T, mut writer: W, color: bool, json: bool) -> io::Result<()> {
    if json {
        writeln!(
            writer,
            "{}",
            json!({
                "type": "error",
                "content": item.to_string(),
            })
        )
    } else if color {
        writeln!(writer, "{}", format_args!("{}: {item}", "error".red()).bold())
    } else {
        writeln!(writer, "error: {item}")
    }
}

pub fn write_warn<T: Display, W: Write>(item: T, mut writer: W, color: bool, json: bool) -> io::Result<()> {
    if json {
        writeln!(
            writer,
            "{}",
            json!({
                "type": "warning",
                "content": item.to_string(),
            })
        )
    } else if color {
        writeln!(writer, "{}", format_args!("{}: {item}", "warning".yellow()).bold())
    } else {
        writeln!(writer, "warning: {item}")
    }
}

pub fn write_item<T: Display, W: Write>(item: T, mut writer: W, json: bool) -> io::Result<()> {
    if json {
        writeln!(
            writer,
            "{}",
            json!({
                "type": "item",
                "content": item.to_string(),
            })
        )
    } else {
        writeln!(writer, "{item}")
    }
}
