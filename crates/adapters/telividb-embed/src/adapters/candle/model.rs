//! One model, loaded and staying loaded.

use super::bert::QuantizedBert;
use super::device::{best_device, device_name};
use super::tokenize;
use super::weights::Weights;
use crate::domain::{ModelId, Pooling};
use crate::error::{Error, Result};
use candle_core::quantized::gguf_file::Content;
use std::fs::File;
use telividb_core::{Dim, Fingerprint};
use telividb_telemetry::{fields, logger, residency};
use tokenizers::Tokenizer;

/// A GGUF model held in memory, with everything needed to run it.
///
/// Resident for the process's lifetime by design (rule 45): there is no unload
/// and no eviction, because a load-run-unload path would defeat the batching
/// that makes GPU inference worth doing in-process at all.
pub struct ResidentModel {
    /// What this model is, by name and by digest.
    pub id: ModelId,
    /// How token states collapse to one vector.
    pub pooling: Pooling,
    encoder: QuantizedBert,
    tokenizer: Tokenizer,
    /// Registration in the shared residency registry, released on drop.
    ///
    /// This is what puts models into the same accounting as indexes and
    /// stores — a model zoo whose GPU footprint is invisible to the index's
    /// budget would let the two overcommit the same device independently.
    _resident: residency::Handle,
}

impl ResidentModel {
    /// Load `path`, verifying it is the file `id` names.
    ///
    /// The digest is computed over the bytes actually read, not trusted from
    /// the caller: rule 12's guarantee is only worth having if the identity is
    /// checked against the file rather than asserted alongside it.
    pub fn load(id: &ModelId, path: &std::path::Path, pooling: Option<Pooling>) -> Result<Self> {
        let bytes = std::fs::read(path)?;
        let found = Fingerprint::of(&bytes);
        if !id.fingerprint.is_unset() && found != id.fingerprint {
            return Err(Error::DigestMismatch {
                name: id.name.clone(),
                expected: id.fingerprint.short(),
                found: found.short(),
            });
        }
        drop(bytes);

        let mut file = File::open(path)?;
        let content = Content::read(&mut file)?;
        let tokenizer = tokenize::from_gguf(&content)?;
        let device = best_device();

        let mut weights = Weights::new(content, file, device.clone());
        let encoder = QuantizedBert::load(&mut weights)?;

        // The caller's choice wins when they made one; otherwise take what the
        // model declares. Mean is the last resort rather than the default,
        // because it is only reached when the file says nothing at all.
        let pooling = pooling
            .or_else(|| encoder.declared_pooling())
            .unwrap_or(Pooling::Mean);

        let resident_bytes = std::fs::metadata(path).map(|m| m.len() as usize)?;
        let location = match device {
            candle_core::Device::Cpu => residency::Location::Host,
            _ => residency::Location::Device,
        };
        let _resident = residency::register(
            residency::ResidentKind::Model,
            location,
            id.name.clone(),
            resident_bytes,
        );

        logger::info!("model resident").with_data(&serde_json::json!({
            fields::MODEL: id.name,
            fields::MODEL_FINGERPRINT: found.short(),
            fields::DEVICE: device_name(&device),
            fields::DIM: encoder.hidden(),
            fields::POOLING: pooling.as_str(),
            fields::RESIDENT_BYTES: resident_bytes,
        }));

        Ok(Self {
            id: ModelId::new(id.name.clone(), found),
            pooling,
            encoder,
            tokenizer,
            _resident,
        })
    }

    /// The width of the vectors this model produces.
    pub fn dim(&self) -> Result<Dim> {
        Dim::new(self.encoder.hidden() as u32).map_err(|_| Error::MissingFromGguf {
            what: format!(
                "a usable embedding width; {} is not one",
                self.encoder.hidden()
            ),
        })
    }

    /// The loaded encoder.
    pub fn encoder(&self) -> &QuantizedBert {
        &self.encoder
    }

    /// The tokenizer built from this model's own GGUF.
    pub fn tokenizer(&self) -> &Tokenizer {
        &self.tokenizer
    }

    /// The architecture this model declares, e.g. `nomic-bert`.
    pub fn architecture(&self) -> &str {
        self.encoder.architecture()
    }
}
