//! Parsing the body of a service, message or enum.
//!
//! Split from file scanning because they answer different questions: one finds
//! where a declaration begins, these read what is inside it. Both track brace
//! depth, and keeping that logic in one place stops the two drifting.

use super::model::{Enum, EnumValue, Field, Message, Rpc, Service};

pub fn parse_service_body<'a>(
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

pub fn parse_message_body<'a>(
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

pub fn parse_enum_body<'a>(
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
