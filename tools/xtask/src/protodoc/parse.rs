//! Reading `.proto` files into the documentation model.
//!
//! A line-oriented parser rather than a descriptor-set reader, for one reason:
//! **comments**. A descriptor set carries source info only when built for it,
//! and the prose above a field is most of what documentation is worth. Reading
//! the text keeps the comments attached to what they describe.
//!
//! It understands the subset this repository actually writes — messages,
//! enums, services, fields, and leading comments — and ignores the rest rather
//! than guessing. Anything it cannot parse simply does not appear, which is a
//! visible gap rather than a wrong one.

use super::blocks::{parse_enum_body, parse_message_body, parse_service_body};
use super::model::{Enum, Message, Module, Service};
use std::path::Path;

/// Parse every `.proto` under a module directory into `module`.
pub fn parse_module(dir: &Path, module: &mut Module) {
    let mut files: Vec<std::path::PathBuf> = Vec::new();
    collect_protos(dir, &mut files);
    files.sort();

    for file in files {
        if let Ok(text) = std::fs::read_to_string(&file) {
            parse_file(&text, module);
        }
    }
}

fn collect_protos(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_protos(&path, out);
        } else if path.extension().is_some_and(|e| e == "proto") {
            out.push(path);
        }
    }
}

/// Parse one file's contents, appending to `module`.
fn parse_file(text: &str, module: &mut Module) {
    let mut comment = String::new();
    let mut lines = text.lines().peekable();

    // `while let` rather than `for`, because the body hands `lines` to the
    // sub-parsers to consume a nested block. A `for` loop would hold the
    // borrow for the whole iteration and make that impossible.
    #[allow(clippy::while_let_on_iterator)]
    while let Some(raw) = lines.next() {
        let line = raw.trim();

        if let Some(rest) = line.strip_prefix("//") {
            let rest = rest.trim();
            if !comment.is_empty() {
                comment.push(' ');
            }
            comment.push_str(rest);
            continue;
        }

        if line.is_empty() {
            comment.clear();
            continue;
        }

        if let Some(pkg) = line.strip_prefix("package ") {
            module.package = pkg.trim_end_matches(';').trim().to_owned();
        } else if let Some(import) = line.strip_prefix("import ") {
            let path = import.trim_matches(|c| c == '"' || c == ';' || c == ' ');
            module.imports.push(path.to_owned());
        } else if let Some(name) = line.strip_prefix("service ") {
            let name = name.split_whitespace().next().unwrap_or("").to_owned();
            let mut service = Service {
                name,
                comment: std::mem::take(&mut comment),
                rpcs: Vec::new(),
            };
            parse_service_body(&mut lines, &mut service);
            module.services.push(service);
        } else if let Some(name) = line.strip_prefix("message ") {
            let name = name.split_whitespace().next().unwrap_or("").to_owned();
            let mut message = Message {
                name,
                comment: std::mem::take(&mut comment),
                fields: Vec::new(),
            };
            parse_message_body(&mut lines, &mut message);
            module.messages.push(message);
        } else if let Some(name) = line.strip_prefix("enum ") {
            let name = name.split_whitespace().next().unwrap_or("").to_owned();
            let mut value = Enum {
                name,
                comment: std::mem::take(&mut comment),
                values: Vec::new(),
            };
            parse_enum_body(&mut lines, &mut value);
            module.enums.push(value);
        }

        comment.clear();
    }
}

#[cfg(test)]
#[path = "parse_test.rs"]
mod tests;
