//! The write-ahead log.
//!
//! Every accepted write lands here before it is visible anywhere else. Records
//! are length-framed with a checksum, and committed in groups so `fsync` is
//! amortised across a batch rather than paid per row.
//!
//! The property that matters on restart: **a torn tail is detected, not
//! trusted.** A process killed mid-write leaves a partial final record; the
//! reader stops cleanly at the last intact one instead of replaying garbage.

mod frame;
mod reader;
mod writer;

pub use frame::FRAME_HEADER_BYTES;
pub use reader::{WalReader, WalTail};
pub use writer::WalWriter;
