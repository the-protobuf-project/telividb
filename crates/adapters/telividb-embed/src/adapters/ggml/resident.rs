//! A model held resident, with its identity checked against the file.
//!
//! Rule 12's guarantee — a vector field is bound to one model identity — is
//! only worth having if the identity is verified against the bytes actually
//! read, rather than asserted alongside them. The digest is computed here on
//! load, not trusted from the caller.

use super::encoder::Encoder;
use crate::domain::{ModelId, Pooling};
use crate::error::{Error, Result};
use std::path::Path;
use telividb_compute::Backend;
use telividb_core::Fingerprint;
use telividb_telemetry::residency;
use tokenizers::Tokenizer;

/// One loaded model: weights on a device, tokenizer, and how it pools.
pub struct ResidentModel {
    /// What this model is, by name and by digest.
    pub id: ModelId,
    /// How token states collapse to one vector.
    pub pooling: Pooling,
    /// Behind a lock because a ggml backend holds one command queue, so two
    /// threads submitting compute to it race. The `Inferencer` port is
    /// `Send + Sync` — concurrent callers are the point, since batching them
    /// together is what keeps a device busy — and this is what makes that
    /// safe. It costs nothing real: a device executes submitted work serially
    /// regardless.
    encoder: std::sync::Mutex<Encoder>,
    tokenizer: Tokenizer,
    /// Registration in the shared residency registry, released on drop.
    ///
    /// What puts models into the same accounting as indexes and stores — a
    /// model zoo whose device footprint were invisible to the index budget
    /// would let the two overcommit the same device independently.
    _resident: residency::Handle,
}

impl ResidentModel {
    /// Load `path` onto `backend`, verifying it is the file `id` names.
    pub fn load(
        id: &ModelId,
        path: &Path,
        backend: Backend,
        pooling: Option<Pooling>,
    ) -> Result<Self> {
        let bytes = std::fs::read(path)?;
        let found = Fingerprint::of(&bytes);
        if !id.fingerprint.is_unset() && id.fingerprint != found {
            return Err(Error::DigestMismatch {
                name: id.name.clone(),
                expected: id.fingerprint.to_string(),
                found: found.to_string(),
            });
        }

        // The tokenizer comes out of the same file as the weights, so a swap
        // cannot change token ids without changing the digest (rule 12).
        let tokenizer = super::vocab::from_gguf(path)?;
        let encoder = Encoder::load(path, backend)?;

        let resident = residency::register(
            residency::ResidentKind::Model,
            residency::Location::Device,
            &id.name,
            bytes.len(),
        );

        Ok(Self {
            id: ModelId::new(id.name.clone(), found),
            pooling: pooling.unwrap_or(Pooling::Mean),
            encoder: std::sync::Mutex::new(encoder),
            tokenizer,
            _resident: resident,
        })
    }

    /// Encode one padded batch, holding the device lock for the call.
    pub fn forward(&self, ids: &[u32], attention: &[u32], rows: usize) -> Result<Vec<Vec<f32>>> {
        self.encoder
            .lock()
            .map_err(|_| Error::Compute("the encoder lock was poisoned".to_owned()))?
            .forward(ids, attention, rows, self.pooling)
    }

    /// The longest sequence this model's positions cover.
    pub fn context(&self) -> Result<usize> {
        Ok(self
            .encoder
            .lock()
            .map_err(|_| Error::Compute("the encoder lock was poisoned".to_owned()))?
            .config()
            .context)
    }

    /// The width of the vectors this model produces, as a checked [`Dim`].
    ///
    /// [`Dim`]: telividb_core::Dim
    pub fn dim(&self) -> Result<telividb_core::Dim> {
        let hidden = self.width()?;
        telividb_core::Dim::new(hidden as u32).map_err(|_| Error::MissingFromGguf {
            what: format!("a usable embedding width; the model reports {hidden}"),
        })
    }

    /// The width of the vectors this model produces.
    pub fn width(&self) -> Result<usize> {
        Ok(self
            .encoder
            .lock()
            .map_err(|_| Error::Compute("the encoder lock was poisoned".to_owned()))?
            .config()
            .hidden)
    }

    /// The tokenizer built from this model's own GGUF.
    pub fn tokenizer(&self) -> &Tokenizer {
        &self.tokenizer
    }
}
