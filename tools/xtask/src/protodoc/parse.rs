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

use super::model::{Enum, EnumValue, Field, Message, Module, Rpc, Service};
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

    for raw in lines.by_ref() {
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

fn parse_service_body<'a>(
    lines: &mut std::iter::Peekable<impl Iterator<Item = &'a str>>,
    service: &mut Service,
) {
    let mut comment = String::new();
    let mut depth = 1usize;

    for raw in lines.by_ref() {
        let line = raw.trim();
        depth = depth.saturating_add(line.matches('{').count());
        depth = depth.saturating_sub(line.matches('}').count());
        if depth == 0 {
            return;
        }

        if let Some(rest) = line.strip_prefix("//") {
            push_comment(&mut comment, rest.trim());
        } else if let Some(rest) = line.strip_prefix("rpc ") {
            service
                .rpcs
                .push(parse_rpc(rest, std::mem::take(&mut comment)));
        } else if !line.is_empty() && !line.starts_with("option") {
            comment.clear();
        }
    }
}

/// `Name(Request) returns (Response) {` or `... returns (stream Response)`.
fn parse_rpc(rest: &str, comment: String) -> Rpc {
    let name = rest.split('(').next().unwrap_or("").trim().to_owned();
    let request = between(rest, '(', ')').unwrap_or_default();
    let response_raw = rest
        .split("returns")
        .nth(1)
        .and_then(|r| between(r, '(', ')'))
        .unwrap_or_default();
    let server_stream = response_raw.starts_with("stream ");

    Rpc {
        name,
        comment,
        request: request.trim().to_owned(),
        response: response_raw.trim_start_matches("stream ").trim().to_owned(),
        server_stream,
    }
}

fn parse_message_body<'a>(
    lines: &mut std::iter::Peekable<impl Iterator<Item = &'a str>>,
    message: &mut Message,
) {
    let mut comment = String::new();
    let mut depth = 1usize;

    for raw in lines.by_ref() {
        let line = raw.trim();
        depth = depth.saturating_add(line.matches('{').count());
        depth = depth.saturating_sub(line.matches('}').count());
        if depth == 0 {
            return;
        }

        if let Some(rest) = line.strip_prefix("//") {
            push_comment(&mut comment, rest.trim());
        } else if let Some(field) = parse_field(line, &comment) {
            message.fields.push(field);
            comment.clear();
        }
    }
}

/// `repeated Type name = 3 [...];`
fn parse_field(line: &str, comment: &str) -> Option<Field> {
    if !line.contains('=') || line.starts_with("option") || line.starts_with("//") {
        return None;
    }
    let (decl, tail) = line.split_once('=')?;
    let parts: Vec<&str> = decl.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }

    let name = parts.last()?.to_string();
    let type_name = parts[..parts.len() - 1].join(" ");
    let number: i32 = tail
        .trim()
        .trim_end_matches(';')
        .split(|c: char| !c.is_ascii_digit())
        .find(|s| !s.is_empty())
        .and_then(|s| s.parse().ok())?;

    let behavior = ["REQUIRED", "OPTIONAL", "OUTPUT_ONLY", "IDENTIFIER"]
        .iter()
        .find(|b| line.contains(*b))
        .map(|b| (*b).to_owned())
        .unwrap_or_default();

    Some(Field {
        name,
        type_name,
        number,
        comment: comment.to_owned(),
        behavior,
    })
}

fn parse_enum_body<'a>(
    lines: &mut std::iter::Peekable<impl Iterator<Item = &'a str>>,
    value: &mut Enum,
) {
    let mut comment = String::new();
    let mut depth = 1usize;

    for raw in lines.by_ref() {
        let line = raw.trim();
        depth = depth.saturating_add(line.matches('{').count());
        depth = depth.saturating_sub(line.matches('}').count());
        if depth == 0 {
            return;
        }

        if let Some(rest) = line.strip_prefix("//") {
            push_comment(&mut comment, rest.trim());
        } else if let Some((name, tail)) = line.split_once('=') {
            let number = tail
                .trim()
                .trim_end_matches(';')
                .trim()
                .parse()
                .unwrap_or(0);
            value.values.push(EnumValue {
                name: name.trim().to_owned(),
                number,
                comment: std::mem::take(&mut comment),
            });
        }
    }
}

fn push_comment(buffer: &mut String, text: &str) {
    if text.is_empty() {
        return;
    }
    if !buffer.is_empty() {
        buffer.push(' ');
    }
    buffer.push_str(text);
}

fn between(text: &str, open: char, close: char) -> Option<String> {
    let start = text.find(open)? + 1;
    let end = text[start..].find(close)? + start;
    Some(text[start..end].to_owned())
}

#[cfg(test)]
#[path = "parse_test.rs"]
mod tests;
