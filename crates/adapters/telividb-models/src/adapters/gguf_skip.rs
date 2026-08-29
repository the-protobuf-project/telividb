//! Stepping over metadata values, and reading the two that matter.
//!
//! Split from `gguf.rs` because that file is about what a header *says* and
//! this one about how to walk past everything it says that nobody here needs.
//! The walking is the fiddly half — a value's width depends on a tag, arrays
//! nest, and getting it wrong does not error: the scan simply stops and every
//! field after that point is silently lost.

use super::gguf::tag;
use super::gguf_reader::Cursor;

/// Whether `kind` is an integer this reader accepts for a shape field.
pub(super) fn is_int(kind: u32) -> bool {
    kind == tag::U32 || kind == tag::U64
}

/// Read a shape field, whichever integer width it was written as.
///
/// A `u64` too large for the field is dropped rather than truncated: a model
/// claiming four billion dimensions is a misread, and silently wrapping it to a
/// small number would be a shape the rest of the code would act on.
pub(super) fn read_int(cursor: &mut Cursor<'_>, kind: u32) -> Option<u32> {
    match kind {
        tag::U64 => u32::try_from(cursor.u64()?).ok(),
        _ => cursor.u32(),
    }
}

/// Step over a metadata value of `kind`, returning false if the buffer ended.
pub(super) fn skip(cursor: &mut Cursor<'_>, kind: u32) -> bool {
    if let Some(width) = width_of(kind) {
        return cursor.take(width).is_some();
    }
    match kind {
        tag::STRING => cursor.string().is_some(),
        tag::ARRAY => skip_array(cursor, 0),
        // An unknown tag means the rest of the header cannot be located, so
        // stop rather than resynchronise on what would be arbitrary bytes.
        _ => false,
    }
}

/// How deeply arrays may nest before the header is treated as malformed.
///
/// The bytes here come from a model host, so the nesting depth is not this
/// crate's to trust: each level costs twelve bytes to declare, and a two-megabyte
/// prefix could therefore declare well over a hundred thousand of them. Without
/// a bound the recursion below would overflow the stack on input that a server
/// is free to send. Eight is far past anything a real file uses.
const MAX_ARRAY_DEPTH: usize = 8;

/// Step over a counted array, including the vocabulary arrays.
pub(super) fn skip_array(cursor: &mut Cursor<'_>, depth: usize) -> bool {
    if depth > MAX_ARRAY_DEPTH {
        return false;
    }
    let (Some(element), Some(count)) = (cursor.u32(), cursor.u64()) else {
        return false;
    };
    if element == tag::STRING {
        // Strings are individually length-prefixed, so this is the one case
        // that cannot be skipped with arithmetic.
        return (0..count).all(|_| cursor.string().is_some());
    }
    if element == tag::ARRAY {
        // Arrays nest, and the specification allows it. Each inner array
        // carries its own element tag and count, so the only way past one is to
        // read it. Rare in practice — but "rare" here meant the scan stopped at
        // the first one, losing every field after it, which for a file that
        // wrote its vocabulary early is the architecture itself.
        return (0..count).all(|_| skip_array(cursor, depth + 1));
    }
    let Some(width) = width_of(element) else {
        return false;
    };
    match usize::try_from(count)
        .ok()
        .and_then(|c| c.checked_mul(width))
    {
        Some(total) => cursor.take(total).is_some(),
        None => false,
    }
}

/// The fixed width of a scalar tag, or `None` for strings and arrays.
pub(super) fn width_of(kind: u32) -> Option<usize> {
    [
        (tag::ONE_BYTE, 1),
        (tag::TWO_BYTE, 2),
        (tag::FOUR_BYTE, 4),
        (tag::EIGHT_BYTE, 8),
    ]
    .into_iter()
    .find_map(|(tags, width)| tags.contains(&kind).then_some(width))
}
