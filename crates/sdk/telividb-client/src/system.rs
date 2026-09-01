//! What the engine found when it started.

use crate::error::Result;
use telividb_buffers::protobuf::system::v1 as wire;

/// How the device memory ceiling was determined.
///
/// Worth carrying rather than flattening into the number: an estimate on a
/// discrete card overshoots, and someone sizing a deployment has to be able to
/// tell which kind of number they are reading.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetSource {
    /// Read from the device.
    Measured,
    /// Inferred, because the backend reports no ceiling.
    Estimated,
    /// Set by configuration, overriding what the device reports.
    Configured,
}

impl BudgetSource {
    /// Read a wire value, treating anything unknown as an estimate.
    ///
    /// An estimate is the weakest claim of the three, so an unrecognised value
    /// degrades to the one that promises least rather than to a measurement
    /// nobody took.
    fn from_wire(value: i32) -> Self {
        match wire::BudgetSource::try_from(value) {
            Ok(wire::BudgetSource::Measured) => Self::Measured,
            Ok(wire::BudgetSource::Configured) => Self::Configured,
            _ => Self::Estimated,
        }
    }

    /// The name used across the IPC boundary and in the window.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Measured => "measured",
            Self::Estimated => "estimated",
            Self::Configured => "configured",
        }
    }
}

/// The compute environment a running engine selected.
#[derive(Debug, Clone)]
pub struct System {
    /// The selected backend: `metal`, `cuda`, `cpu`, and so on.
    ///
    /// The one fact no orchestrator sees from outside: a process that fell back
    /// to the host is healthy, allocated and idle from every angle but this one.
    pub backend: String,
    /// Human-readable device description.
    pub device: String,
    /// Device memory ceiling this process will use. Zero when none is reported.
    pub budget_limit_bytes: i64,
    /// Device memory currently held by resident models and indexes.
    pub budget_used_bytes: i64,
    /// Whether the ceiling was measured, estimated, or configured.
    pub budget_source: BudgetSource,
    /// Version of the engine build that answered.
    pub version: String,
}

impl From<wire::System> for System {
    fn from(s: wire::System) -> Self {
        Self {
            backend: s.backend,
            device: s.device,
            budget_limit_bytes: s.budget_limit_bytes,
            budget_used_bytes: s.budget_used_bytes,
            budget_source: BudgetSource::from_wire(s.budget_source),
            version: s.version,
        }
    }
}

impl crate::Client {
    /// Describe the engine's compute environment.
    ///
    /// Asked of the server rather than detected locally, which is what lets a
    /// browser talking to a remote daemon see the same answer the desktop app
    /// does. A client that detected its own hardware would be describing the
    /// wrong machine entirely.
    pub async fn system(&self) -> Result<System> {
        let system = self
            .system
            .clone()
            .get_system(wire::GetSystemRequest {
                name: "system".to_owned(),
            })
            .await?
            .into_inner();
        Ok(system.into())
    }
}
