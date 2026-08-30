//! The `SystemInfo` service.

use telividb_buffers::protobuf::system::v1 as wire;
use telividb_buffers::protobuf::system::v1::system_info_server::SystemInfo;
use tonic::{Request, Response, Status};

/// The singleton resource name this service answers for.
///
/// One machine, one process, one answer — so the resource has a fixed name
/// rather than an id. A caller asking for anything else has the wrong server.
const NAME: &str = "system";

/// Reports the compute environment this process selected.
///
/// **Every number here comes from the index's budget accounting, not from a
/// fresh reading of the device.** The two differ and the difference matters: the
/// device's total is what it can address, while the budget is what this process
/// will actually let resident indexes hold, and residency is what the shared
/// registry says is reserved rather than what the driver has allocated. Reporting
/// the device's own figures would describe a machine rather than this engine.
#[derive(Clone, Default)]
pub struct SystemSvc;

impl SystemSvc {
    /// A service reporting on this process.
    pub fn new() -> Self {
        Self
    }

    /// Describe the environment, reading the budget in force.
    ///
    /// Measured at call time rather than cached at startup: residency changes as
    /// indexes are built and dropped, and a cached figure would go on describing
    /// a corpus that had since been freed.
    #[cfg(feature = "gpu")]
    fn describe(&self) -> wire::System {
        use telividb_index::adapters::{
            BudgetSource, Device, budget_source, device_name, gpu_budget_bytes, gpu_resident_bytes,
        };

        let kind = Device::best().kind();
        wire::System {
            name: NAME.to_owned(),
            backend: kind.as_str().to_owned(),
            // What ggml opened, which is not always what was asked for.
            device: device_name().unwrap_or_else(|| "unavailable".to_owned()),
            budget_limit_bytes: i64::try_from(gpu_budget_bytes()).unwrap_or(i64::MAX),
            budget_used_bytes: i64::try_from(gpu_resident_bytes()).unwrap_or(i64::MAX),
            budget_source: match budget_source() {
                BudgetSource::Configured => wire::BudgetSource::Configured,
                BudgetSource::DeviceReported => wire::BudgetSource::Measured,
                BudgetSource::Estimated => wire::BudgetSource::Estimated,
            } as i32,
            version: env!("CARGO_PKG_VERSION").to_owned(),
        }
    }

    /// Describe a build with no device path compiled in.
    ///
    /// Says so, rather than reporting `cpu` as though it had looked. "Built
    /// without the gpu feature" and "looked, and found only a host backend" are
    /// different facts, and an operator debugging slow search needs the true one.
    #[cfg(not(feature = "gpu"))]
    fn describe(&self) -> wire::System {
        wire::System {
            name: NAME.to_owned(),
            backend: "cpu".to_owned(),
            device: "built without the gpu feature; no device was queried".to_owned(),
            budget_limit_bytes: 0,
            budget_used_bytes: 0,
            budget_source: wire::BudgetSource::Estimated as i32,
            version: env!("CARGO_PKG_VERSION").to_owned(),
        }
    }
}

#[tonic::async_trait]
impl SystemInfo for SystemSvc {
    /// Describe this process's compute environment.
    async fn get_system(
        &self,
        request: Request<wire::GetSystemRequest>,
    ) -> Result<Response<wire::System>, Status> {
        let asked = &request.get_ref().name;
        if !asked.is_empty() && asked != NAME {
            return Err(Status::not_found(format!(
                "{asked:?} not found. This server describes one machine, and its \
                 resource name is {NAME:?}."
            )));
        }
        Ok(Response::new(self.describe()))
    }
}

#[cfg(test)]
#[path = "info_test.rs"]
mod tests;
