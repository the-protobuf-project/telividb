//! Starting, following and cancelling an installation.

use super::{ModelsSvc, catalog_id, catalog_name};
use crate::services::clock::{now_millis, now_stamp, stamp};
use telividb_buffers::protobuf::models::v1::{
    CreateModelInstallationRequest, DeleteModelInstallationRequest, GetModelInstallationRequest,
    InstallationState, ListModelInstallationsRequest, ListModelInstallationsResponse,
    ModelInstallation,
};
use telividb_models::{CatalogEntry, HttpFetcher, ModelStore};
use telividb_telemetry::logger;
use tonic::{Request, Response, Status};

impl ModelsSvc {
    /// Begin installing a catalog model.
    ///
    /// Returns once the work is accepted, never once it finishes.
    pub(super) fn create_install(
        &self,
        request: Request<CreateModelInstallationRequest>,
    ) -> Result<Response<ModelInstallation>, Status> {
        let req = request.into_inner();
        let requested = req
            .model_installation
            .ok_or_else(|| Status::invalid_argument("model_installation is required"))?;

        let id = catalog_id(&requested.catalog_model);
        let entry = self
            .catalog
            .get(id)
            .ok_or_else(|| {
                Status::not_found(format!(
                    "no catalog model called {:?}",
                    requested.catalog_model
                ))
            })?
            .clone();

        // The id defaults to the model's, which makes repeating the call
        // idempotent by default: asking twice for the same model returns the
        // running installation rather than starting a second transfer of one
        // file into one path.
        let install_id = match req.model_installation_id.is_empty() {
            true => entry.id.clone(),
            false => req.model_installation_id,
        };
        let name = format!("modelInstallations/{install_id}");

        let mut installs = self.installs()?;
        if let Some(existing) = installs.get(&name) {
            return Ok(Response::new(existing.clone()));
        }

        let now = now_millis();
        let record = ModelInstallation {
            name: name.clone(),
            catalog_model: catalog_name(&entry.id),
            state: InstallationState::Pending as i32,
            progress_bytes: 0,
            total_bytes: entry.size_bytes as i64,
            error: String::new(),
            create_time: Some(stamp(now)),
            update_time: Some(stamp(now)),
        };
        installs.insert(name.clone(), record.clone());
        drop(installs);

        self.spawn(name, entry);
        Ok(Response::new(record))
    }

    /// Run the transfer on a blocking thread.
    ///
    /// Blocking, not async: the fetcher is synchronous and a download is IO
    /// that would otherwise sit on an executor thread for minutes, which
    /// invariant 5 forbids for exactly this reason.
    fn spawn(&self, name: String, entry: CatalogEntry) {
        let store = self.store.clone();
        let installs = self.installs.clone();
        tokio::task::spawn_blocking(move || {
            let outcome = install(&store, &entry, &name, &installs);
            let mut guard = match installs.lock() {
                Ok(guard) => guard,
                Err(_) => return,
            };
            let Some(record) = guard.get_mut(&name) else {
                return;
            };
            record.update_time = Some(now_stamp());
            match outcome {
                Ok(()) => {
                    record.state = InstallationState::Succeeded as i32;
                    record.progress_bytes = record.total_bytes;
                }
                Err(telividb_models::Error::Cancelled { written, .. }) => {
                    record.state = InstallationState::Cancelled as i32;
                    record.progress_bytes = written as i64;
                }
                Err(e) => {
                    record.state = InstallationState::Failed as i32;
                    record.error = e.to_string();
                }
            }
        });
    }

    /// An installation by name.
    pub(super) fn get_install(
        &self,
        request: Request<GetModelInstallationRequest>,
    ) -> Result<Response<ModelInstallation>, Status> {
        let name = request.into_inner().name;
        self.installs()?
            .get(&name)
            .cloned()
            .map(Response::new)
            .ok_or_else(|| Status::not_found(format!("no installation called {name:?}")))
    }

    /// Every installation this process knows about, most recent first.
    pub(super) fn list_installs(
        &self,
        _request: Request<ListModelInstallationsRequest>,
    ) -> Result<Response<ListModelInstallationsResponse>, Status> {
        let mut model_installations: Vec<ModelInstallation> =
            self.installs()?.values().cloned().collect();
        model_installations.sort_by_key(|i| std::cmp::Reverse(i.create_time.map(|t| t.seconds)));
        Ok(Response::new(ListModelInstallationsResponse {
            model_installations,
            next_page_token: String::new(),
        }))
    }

    /// Cancel a running installation, or forget a finished one.
    pub(super) fn delete_install(
        &self,
        request: Request<DeleteModelInstallationRequest>,
    ) -> Result<Response<ModelInstallation>, Status> {
        let name = request.into_inner().name;
        let mut installs = self.installs()?;
        let record = installs
            .get_mut(&name)
            .ok_or_else(|| Status::not_found(format!("no installation called {name:?}")))?;

        // Marking it is the cancel: the transfer reads this between chunks and
        // stops, keeping its partial file so installing again resumes.
        if record.state == InstallationState::Pending as i32
            || record.state == InstallationState::Downloading as i32
        {
            record.state = InstallationState::Cancelled as i32;
        }
        Ok(Response::new(record.clone()))
    }
}

/// The transfer itself, on a blocking thread.
fn install(
    store: &ModelStore,
    entry: &CatalogEntry,
    name: &str,
    installs: &super::Registry,
) -> Result<(), telividb_models::Error> {
    logger::info!("model install started").with_data(&serde_json::json!({
        "telividb.model.id": entry.id,
        "telividb.model.bytes": entry.size_bytes,
    }));

    let fetcher = HttpFetcher::new()?;
    store
        .install(entry, &fetcher, &mut |written| {
            let Ok(mut guard) = installs.lock() else {
                return false;
            };
            let Some(record) = guard.get_mut(name) else {
                return false;
            };
            // A delete sets `Cancelled` while this runs; seeing it here is how
            // the transfer learns to stop.
            if record.state == InstallationState::Cancelled as i32 {
                return false;
            }
            record.state = InstallationState::Downloading as i32;
            record.progress_bytes = written as i64;
            true
        })
        .map(|_| ())
}
