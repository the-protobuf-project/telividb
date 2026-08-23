//! Resource name templates — AIP-122 style.

use super::ResourceName;
use crate::error::{Error, Result};

/// A pattern such as `collections/{collection}/points/{point}`.
///
/// Each `{placeholder}` matches exactly one segment and never spans a `/`.
/// Compile once, then parse or format repeatedly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Template {
    segments: Vec<Segment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Segment {
    Literal(String),
    Placeholder(String),
}

impl Template {
    pub fn compile(pattern: &str) -> Result<Self> {
        let segments = pattern
            .split('/')
            .map(
                |seg| match seg.strip_prefix('{').and_then(|s| s.strip_suffix('}')) {
                    Some(key) if !key.is_empty() => Ok(Segment::Placeholder(key.to_owned())),
                    Some(_) => Err(Error::InvalidResourceName {
                        name: pattern.to_owned(),
                        reason: "empty placeholder",
                    }),
                    None if seg.is_empty() => Err(Error::InvalidResourceName {
                        name: pattern.to_owned(),
                        reason: "empty segment",
                    }),
                    None => Ok(Segment::Literal(seg.to_owned())),
                },
            )
            .collect::<Result<Vec<_>>>()?;
        Ok(Self { segments })
    }

    /// Placeholder names, in the order they appear.
    pub fn placeholders(&self) -> impl Iterator<Item = &str> {
        self.segments.iter().filter_map(|s| match s {
            Segment::Placeholder(key) => Some(key.as_str()),
            Segment::Literal(_) => None,
        })
    }

    /// Extract placeholder bindings from `name`, or `None` if it does not match.
    pub fn parse<'a>(&self, name: &'a ResourceName) -> Option<Vec<(&str, &'a str)>> {
        let mut have = name.segments();
        let mut bound = Vec::new();

        for want in &self.segments {
            let seg = have.next()?;
            match want {
                Segment::Literal(lit) if lit == seg => {}
                Segment::Literal(_) => return None,
                Segment::Placeholder(key) => bound.push((key.as_str(), seg)),
            }
        }
        if have.next().is_some() {
            return None;
        }
        Some(bound)
    }

    /// Build a name by substituting `values` in placeholder order.
    pub fn format(&self, values: &[&str]) -> Result<ResourceName> {
        let mut next = values.iter();
        let mut out = String::new();

        for seg in &self.segments {
            if !out.is_empty() {
                out.push('/');
            }
            match seg {
                Segment::Literal(lit) => out.push_str(lit),
                Segment::Placeholder(key) => {
                    let value = next.next().ok_or(Error::InvalidResourceName {
                        name: key.clone(),
                        reason: "missing value for placeholder",
                    })?;
                    if value.contains('/') {
                        return Err(Error::InvalidResourceName {
                            name: (*value).to_owned(),
                            reason: "a placeholder value may not contain '/'",
                        });
                    }
                    out.push_str(value);
                }
            }
        }
        ResourceName::parse(out)
    }
}

#[cfg(test)]
#[path = "template_test.rs"]
mod tests;
