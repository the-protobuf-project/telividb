//! What the engine reports about the machine it is running on.

use serde::Serialize;

/// The compute environment, as the engine selected it.
///
/// Asked of the engine over gRPC rather than detected in this process. The
/// desktop app could detect it locally only because it links the engine; a
/// browser talking to a Linux daemon cannot, and the two deployments must reach
/// the same answer. This is that answer.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemDto {
    /// The selected backend: `metal`, `cuda`, `cpu`, and so on.
    ///
    /// The one fact no orchestrator sees from outside the process: a build that
    /// fell back to the host is healthy, allocated and idle from every angle.
    pub backend: String,
    /// Human-readable device description.
    pub device: String,
    /// Device memory ceiling this process will use. Zero when none is reported.
    pub budget_limit_bytes: i64,
    /// Device memory held by resident models and indexes.
    ///
    /// Zero until the engine tracks it — rule 45 leaves multi-model budgeting
    /// open, and zero is honest about not knowing where the device's own used
    /// figure would credit this process with every other process's allocations.
    pub budget_used_bytes: i64,
    /// `measured`, `estimated` or `configured`.
    pub budget_source: String,
    /// Version of the engine build that answered.
    pub version: String,
}

impl From<telividb_client::System> for SystemDto {
    fn from(s: telividb_client::System) -> Self {
        Self {
            backend: s.backend,
            device: s.device,
            budget_limit_bytes: s.budget_limit_bytes,
            budget_used_bytes: s.budget_used_bytes,
            budget_source: s.budget_source.as_str().to_owned(),
            version: s.version,
        }
    }
}
