//! What a documented protobuf module contains.
//!
//! Deliberately a plain data model with no rendering in it. Parsing fills these
//! in and rendering reads them, so a change to the Markdown never risks
//! changing what was understood from the source.

/// A directory whose version sub-directories hold `.proto` files.
///
/// `protobuf/telividb/collection/` is a module; `v1/` inside it is a version.
#[derive(Debug, Default, Clone)]
pub struct Module {
    /// Display name, derived from the directory, e.g. `Collection`.
    pub name: String,
    /// Protobuf package, e.g. `telividb.collection.v1`.
    pub package: String,
    /// Path to the module directory, relative to the protobuf root.
    pub dir: String,
    /// Import paths gathered from every file, used to draw the module graph.
    pub imports: Vec<String>,
    pub services: Vec<Service>,
    pub messages: Vec<Message>,
    pub enums: Vec<Enum>,
}

impl Module {
    /// Modules this one imports from, excluding well-known and vendored types.
    ///
    /// Only local edges are drawn: an arrow to `google.protobuf` on every node
    /// would say nothing about the shape of this API.
    pub fn local_imports(&self, root_prefix: &str) -> Vec<String> {
        let mut out: Vec<String> = self
            .imports
            .iter()
            .filter(|i| i.starts_with(root_prefix))
            .filter_map(|i| {
                let rest = i.strip_prefix(root_prefix)?;
                rest.split('/').next().map(str::to_owned)
            })
            .filter(|m| !m.is_empty() && *m != self.short_name())
            .collect();
        out.sort();
        out.dedup();
        out
    }

    /// Directory name of this module, e.g. `collection`.
    pub fn short_name(&self) -> String {
        self.dir.rsplit('/').next().unwrap_or(&self.dir).to_owned()
    }
}

/// A gRPC service.
#[derive(Debug, Default, Clone)]
pub struct Service {
    pub name: String,
    /// Leading comment, with `//` markers stripped.
    pub comment: String,
    pub rpcs: Vec<Rpc>,
}

/// One method on a service.
#[derive(Debug, Default, Clone)]
pub struct Rpc {
    pub name: String,
    pub comment: String,
    /// Request message type, as written.
    pub request: String,
    /// Response message type, as written.
    pub response: String,
    /// Whether the response is a stream.
    pub server_stream: bool,
}

/// A protobuf message.
#[derive(Debug, Default, Clone)]
pub struct Message {
    pub name: String,
    pub comment: String,
    pub fields: Vec<Field>,
}

/// One field of a message.
#[derive(Debug, Default, Clone)]
pub struct Field {
    pub name: String,
    /// Declared type, including `repeated` where present.
    pub type_name: String,
    pub number: i32,
    pub comment: String,
    /// `REQUIRED`, `OPTIONAL`, `OUTPUT_ONLY` or `IDENTIFIER`, when annotated.
    ///
    /// Surfaced because it is the difference between a field a caller must set
    /// and one the server fills in — the single most useful thing to know when
    /// reading an API for the first time.
    pub behavior: String,
}

/// A protobuf enum.
#[derive(Debug, Default, Clone)]
pub struct Enum {
    pub name: String,
    pub comment: String,
    pub values: Vec<EnumValue>,
}

/// One value of an enum.
#[derive(Debug, Default, Clone)]
pub struct EnumValue {
    pub name: String,
    pub number: i32,
    pub comment: String,
}
