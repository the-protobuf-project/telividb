//! Whether a segment written under one schema is readable under another.
//!
//! A fingerprint mismatch is not automatically fatal. Schemas evolve, and
//! protobuf already defines which evolutions are safe — so a segment whose
//! fingerprint differs is **readable when the difference is additive** and
//! refused otherwise.
//!
//! Protobuf's compatibility discipline and immutable segments turn out to be
//! the same discipline:
//!
//! | Change | Verdict | Why |
//! |---|---|---|
//! | New point type, new vector field | **Additive** | Older segments simply lack it; the presence bitmap already models a point that lacks a modality |
//! | Field removed, point type removed | **Breaking** | Rows exist that the current schema cannot describe |
//! | Dimension or metric changed | **Breaking** | Fixed stride and ranking both assume they did not |
//! | Model changed | **Breaking** | A new model is a *new field*, never a mutation — which is what keeps immutability honest |

use super::CollectionSchema;

/// The result of comparing the schema a segment was written under against the
/// collection's current schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Compatibility {
    /// Same fingerprint. Nothing to check.
    Identical,
    /// Readable. The current schema only adds to what the segment holds.
    Additive {
        /// Point types the current schema has that the segment does not.
        added_point_types: Vec<String>,
        /// Vector fields added to a point type the segment already knew.
        added_fields: Vec<String>,
    },
    /// Not readable. Each reason names one incompatibility.
    Breaking {
        /// Every incompatibility found, not merely the first.
        reasons: Vec<String>,
    },
}

impl Compatibility {
    /// Whether a segment under `self` may be read.
    pub fn is_readable(&self) -> bool {
        !matches!(self, Compatibility::Breaking { .. })
    }
}

/// Compare the schema a segment was written under against the current one.
pub fn compare(segment: &CollectionSchema, current: &CollectionSchema) -> Compatibility {
    if segment.fingerprint == current.fingerprint && !segment.fingerprint.is_unset() {
        return Compatibility::Identical;
    }

    let mut reasons = Vec::new();
    let mut added_fields = Vec::new();

    for old_type in &segment.point_types {
        let Some(new_type) = current.point_type(&old_type.type_name) else {
            reasons.push(format!(
                "point type {} was removed; segment holds rows the current schema cannot describe",
                old_type.type_name
            ));
            continue;
        };

        for old_field in &old_type.vector_fields {
            let Some(new_field) = new_type.vector_field(&old_field.name) else {
                reasons.push(format!(
                    "vector field {}.{} was removed",
                    old_type.type_name, old_field.name
                ));
                continue;
            };
            check_field(&old_type.type_name, old_field, new_field, &mut reasons);
        }
    }

    // Anything the current schema has that the segment did not is additive.
    let added_point_types: Vec<String> = current
        .point_types
        .iter()
        .filter(|p| segment.point_type(&p.type_name).is_none())
        .map(|p| p.type_name.clone())
        .collect();

    for new_type in &current.point_types {
        let Some(old_type) = segment.point_type(&new_type.type_name) else {
            continue;
        };
        for new_field in &new_type.vector_fields {
            if old_type.vector_field(&new_field.name).is_none() {
                added_fields.push(format!("{}.{}", new_type.type_name, new_field.name));
            }
        }
    }

    if !reasons.is_empty() {
        return Compatibility::Breaking { reasons };
    }
    Compatibility::Additive {
        added_point_types,
        added_fields,
    }
}

fn check_field(
    type_name: &str,
    old: &super::VectorFieldSpec,
    new: &super::VectorFieldSpec,
    reasons: &mut Vec<String>,
) {
    if old.dim != new.dim {
        reasons.push(format!(
            "{type_name}.{} changed dimension {} -> {}; stride and every stored row assume otherwise",
            old.name,
            old.dim.get(),
            new.dim.get()
        ));
    }
    if old.metric != new.metric {
        reasons.push(format!(
            "{type_name}.{} changed metric {:?} -> {:?}; stored vectors were normalised for the old one",
            old.name, old.metric, new.metric
        ));
    }
    if !old.model_fingerprint.is_unset()
        && !new.model_fingerprint.is_unset()
        && old.model_fingerprint != new.model_fingerprint
    {
        reasons.push(format!(
            "{type_name}.{} changed model {} -> {}; a new model is a new field, not a mutation",
            old.name,
            old.model_fingerprint.short(),
            new.model_fingerprint.short()
        ));
    }
}

#[cfg(test)]
#[path = "compat_test.rs"]
mod tests;
