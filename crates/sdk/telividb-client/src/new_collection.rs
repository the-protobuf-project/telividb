//! Declaring a collection before anything is written to it.

use telividb_buffers::protobuf::collection::v1 as wire;

/// How similarity is measured in a vector field.
///
/// Declared per field and fixed for its life. It decides what "nearest" means,
/// so a field created under the wrong one ranks correctly-stored vectors
/// wrongly — without any error, at any point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Metric {
    /// Angular similarity. What text embedding models want.
    ///
    /// Stored normalised and scored as a dot product, so it costs no more than
    /// [`Metric::Dot`] at query time.
    Cosine,
    /// Inner product. Higher is nearer.
    Dot,
    /// Squared Euclidean distance. Lower is nearer.
    L2,
}

impl Metric {
    /// The wire discriminant.
    fn to_wire(self) -> i32 {
        let value = match self {
            Metric::Cosine => wire::Metric::Cosine,
            Metric::Dot => wire::Metric::Dot,
            Metric::L2 => wire::Metric::L2,
        };
        value as i32
    }
}

/// A collection to create, and the vector fields its points will carry.
///
/// Fields are declared here rather than inferred from the first write. Left to
/// inference, a field's width and model become whatever the first vector
/// happened to be — and a later writer disagreeing is either rejected for the
/// wrong reason or, worse, merged in (see CLAUDE.md rule 12).
#[derive(Debug, Clone)]
pub struct NewCollection {
    pub(crate) id: String,
    pub(crate) descriptor_set: Vec<u8>,
    pub(crate) fields: Vec<wire::VectorField>,
}

impl NewCollection {
    /// Begin declaring a collection.
    ///
    /// The descriptor set is required, not optional: the engine never parses
    /// `.proto`, it consumes compiled bytes, and their digest is the schema's
    /// identity. Produce them with `buf build -o` or
    /// `protoc --descriptor_set_out`.
    pub fn new(id: impl Into<String>, descriptor_set: Vec<u8>) -> Self {
        Self {
            id: id.into(),
            descriptor_set,
            fields: Vec::new(),
        }
    }

    /// Declare a vector field.
    pub fn field(mut self, name: impl Into<String>, dimensions: usize, metric: Metric) -> Self {
        self.fields.push(wire::VectorField {
            field_id: name.into(),
            dimensions: dimensions as i32,
            metric: metric.to_wire(),
            index_kind: wire::IndexKind::Unspecified as i32,
            codec: wire::Codec::Unspecified as i32,
            model: String::new(),
            query_encoder: String::new(),
            permission: String::new(),
        });
        self
    }

    /// Declare a text-embedding field: cosine, at the model's width.
    ///
    /// A named shorthand because it is what nearly every collection wants, and
    /// because cosine is the metric text embedding models are trained for —
    /// picking `L2` here is a quiet accuracy loss rather than an error.
    pub fn text_field(self, name: impl Into<String>, dimensions: usize) -> Self {
        self.field(name, dimensions, Metric::Cosine)
    }

    /// The request payload.
    pub(crate) fn to_wire(&self) -> wire::Collection {
        wire::Collection {
            name: crate::names::collection(&self.id),
            descriptor_set: self.descriptor_set.clone().into(),
            vector_fields: self.fields.clone(),
            ..Default::default()
        }
    }
}
