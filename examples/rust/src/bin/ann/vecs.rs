//! Reading the `.fvecs` / `.ivecs` format the ANN datasets ship in.
//!
//! Each record is a little-endian `int32` dimension followed by that many
//! `f32` (`.fvecs`) or `i32` (`.ivecs`) values. The dimension repeats on every
//! record, which is redundant but is what makes the format self-describing
//! without a header.
//!
//! Parsed here rather than read from the HDF5 the ann-benchmarks site
//! distributes: HDF5 means linking a C library, and invariant 1 allows exactly
//! two native paths — neither of them this. The format needs no library.

use std::path::{Path, PathBuf};

/// Where `examples/datasets/download.sh` puts its files.
pub fn datasets_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(|p| p.join("datasets"))
        .unwrap_or_else(|| PathBuf::from("datasets"))
}

/// A dataset's base vectors, queries, and exact nearest neighbours.
pub struct Dataset {
    /// The corpus to index.
    pub base: Vec<Vec<f32>>,
    /// Query vectors.
    pub queries: Vec<Vec<f32>>,
    /// For each query, the true nearest neighbours by row, nearest first.
    ///
    /// Computed exhaustively by the dataset's authors, so recall measured
    /// against it is exact rather than relative to another approximation.
    pub truth: Vec<Vec<u32>>,
    /// Vector width.
    pub dim: usize,
}

/// Load `name` from the datasets directory.
pub fn load(name: &str) -> Result<Dataset, String> {
    let dir = datasets_dir().join(name);
    let base_path = dir.join(format!("{name}_base.fvecs"));
    if !base_path.exists() {
        return Err(format!(
            "no {name} dataset in {}.\n\nFetch it with:\n    {}/download.sh {name}",
            dir.display(),
            datasets_dir().display()
        ));
    }

    let base = read_fvecs(&base_path)?;
    let mut queries = read_fvecs(&dir.join(format!("{name}_query.fvecs")))?;
    let mut truth = read_ivecs(&dir.join(format!("{name}_groundtruth.ivecs")))?;

    // Capped, because the exhaustive baselines scale with queries x rows: on
    // SIFT-1M all 10,000 queries would take the flat index roughly ten minutes
    // to answer, and every configuration must answer the *same* queries for
    // the comparison to mean anything. A thousand is far more than recall@10
    // needs to be stable, and the cap is raised by setting TELIVIDB_QUERIES.
    let cap = query_cap();
    if queries.len() > cap {
        queries.truncate(cap);
        truth.truncate(cap);
    }

    let dim = base.first().map(Vec::len).unwrap_or(0);
    if dim == 0 {
        return Err(format!("{name} has no vectors"));
    }
    // A query of a different width cannot be scored against this corpus, and
    // would otherwise fail much later inside the index.
    if let Some(bad) = queries.iter().find(|q| q.len() != dim) {
        return Err(format!(
            "{name}: a query has {} dimensions but the corpus has {dim}",
            bad.len()
        ));
    }

    Ok(Dataset {
        base,
        queries,
        truth,
        dim,
    })
}

/// How many queries to measure with.
fn query_cap() -> usize {
    std::env::var("TELIVIDB_QUERIES")
        .ok()
        .and_then(|raw| raw.parse().ok())
        .filter(|n| *n > 0)
        .unwrap_or(1_000)
}

/// Read a `.fvecs` file.
pub fn read_fvecs(path: &Path) -> Result<Vec<Vec<f32>>, String> {
    read_records(path, |bytes| {
        f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    })
}

/// Read an `.ivecs` file, as unsigned row numbers.
pub fn read_ivecs(path: &Path) -> Result<Vec<Vec<u32>>, String> {
    read_records(path, |bytes| {
        u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    })
}

/// Read the repeated `(dim, values…)` records.
///
/// Every read is bounds-checked: the dimension comes from the file itself, so
/// a truncated or corrupt one would otherwise index past the end and panic.
fn read_records<T>(path: &Path, decode: fn(&[u8]) -> T) -> Result<Vec<Vec<T>>, String> {
    let raw = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;

    let mut out = Vec::new();
    let mut offset = 0usize;
    while offset + 4 <= raw.len() {
        let dim = u32::from_le_bytes([
            raw[offset],
            raw[offset + 1],
            raw[offset + 2],
            raw[offset + 3],
        ]) as usize;
        offset += 4;

        let end = offset
            .checked_add(dim * 4)
            .ok_or_else(|| format!("{}: record length overflows", path.display()))?;
        if end > raw.len() {
            return Err(format!(
                "{}: truncated record — declares {dim} values, {} bytes remain",
                path.display(),
                raw.len() - offset
            ));
        }

        out.push(
            raw[offset..end]
                .chunks_exact(4)
                .map(decode)
                .collect::<Vec<T>>(),
        );
        offset = end;
    }
    Ok(out)
}
