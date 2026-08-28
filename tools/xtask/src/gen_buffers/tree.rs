//! Discovering the generated files and normalising where they sit.
//!
//! Each compiler lays its output out differently and neither layout is quite
//! what a reader wants. `flatc` mirrors the schema tree faithfully. `capnpc`
//! writes the package path a second time inside the per-package output
//! directory it was already given, so every file lands at `<pkg>/<pkg>/<file>`.
//!
//! Both are normalised to one tree mirroring the protos, because that is the
//! layout someone looking for `collection_capnp.rs` will look in.

use std::path::{Path, PathBuf};

/// One generated module: what it is called, where it lives, what it belongs to.
pub struct GeneratedModule {
    /// Module name, which is the file stem — `collection_capnp`.
    pub name: String,
    /// Path relative to `generated/<format>/`, used in the `#[path]` attribute.
    pub path: String,
    /// Package segments the facade nests it under — `["collection", "v1"]`.
    pub package: Vec<String>,
}

/// One protobuf package: the module path and the files included into it.
///
/// Protobuf is the odd one out and gets to be tidier for it. `prost` addresses
/// siblings relatively, so these nest properly under `protobuf::<pkg>::v1`
/// instead of needing a flat root and a facade over it.
pub struct ProtoPackage {
    /// Module segments beneath `protobuf` — `["collection", "v1"]`.
    pub package: Vec<String>,
    /// Paths, relative to `src/`, to `include!` into that module.
    pub includes: Vec<String>,
}

/// Every generated module, by target.
pub struct Modules {
    /// Modules rendered from the Cap'n Proto schema. Always populated:
    /// the target is on by default because it generates no `unsafe`.
    pub capnp: Vec<GeneratedModule>,
    /// Modules rendered from the FlatBuffers schema. Rendered even when the
    /// feature is off, so the committed tree does not depend on which
    /// features happened to be enabled on the machine that generated it.
    pub flatbuffers: Vec<GeneratedModule>,
    /// Packages rendered from protobuf by `buf`, nested rather than flattened.
    pub protobuf: Vec<ProtoPackage>,
}

/// Normalise both targets' output and report what was found.
pub fn discover(generated: &Path) -> std::io::Result<Modules> {
    Ok(Modules {
        capnp: normalise(&generated.join("capnp"), "_capnp.rs")?,
        flatbuffers: normalise(&generated.join("flatbuffers"), "_generated.rs")?,
        protobuf: protobuf(&generated.join("protobuf"))?,
    })
}

/// Lift one target's files out of `rust/`, undouble its directories, and index.
fn normalise(base: &Path, suffix: &str) -> std::io::Result<Vec<GeneratedModule>> {
    // The compilers write under a `rust/` subdirectory naming the language.
    // The format directory already says which format this is, so the extra
    // level carries nothing.
    let staged = base.join("rust");
    let mut found = vec![];
    collect(
        if staged.is_dir() { &staged } else { base },
        suffix,
        &mut found,
    )?;

    let mut modules = vec![];
    for src in found {
        let rel = src.strip_prefix(&staged).unwrap_or(&src).to_path_buf();
        let dir = undouble(rel.parent().unwrap_or(Path::new("")));
        let file = rel.file_name().expect("file has a name").to_owned();
        let dest_dir = base.join(&dir);
        std::fs::create_dir_all(&dest_dir)?;
        let dest = dest_dir.join(&file);
        if src != dest {
            std::fs::rename(&src, &dest)?;
        }
        let stem = dest.file_stem().expect("file has a stem").to_string_lossy();
        modules.push(GeneratedModule {
            name: stem.into_owned(),
            path: dir.join(&file).to_string_lossy().replace('\\', "/"),
            package: package_of(&dir),
        });
    }
    prune(base)?;
    modules.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(modules)
}

/// Collapse `a/b/a/b` to `a/b`.
///
/// `capnpc` is invoked once per package with an output directory that already
/// ends in that package's path, and then writes the path again beneath it. The
/// halves are always identical, so an exact match is a safe test — a genuine
/// directory that happened to repeat would have to repeat perfectly.
fn undouble(dir: &Path) -> PathBuf {
    let parts: Vec<_> = dir.iter().collect();
    if !parts.is_empty() && parts.len() % 2 == 0 {
        let (front, back) = parts.split_at(parts.len() / 2);
        if front == back {
            return front.iter().collect();
        }
    }
    dir.to_path_buf()
}

/// Segments the facade nests a module under.
///
/// The leading `telividb` is dropped: the crate is already telividb, and
/// `telividb_buffers::capnp::telividb::collection` says it twice.
fn package_of(dir: &Path) -> Vec<String> {
    dir.iter()
        .map(|s| s.to_string_lossy().into_owned())
        .skip_while(|s| s == "telividb")
        .collect()
}

/// Gather every file under `dir` whose name ends with `suffix`.
pub fn collect(dir: &Path, suffix: &str, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in std::fs::read_dir(dir)?.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, suffix, out)?;
        } else if path.to_string_lossy().ends_with(suffix) {
            out.push(path);
        }
    }
    Ok(())
}

/// Remove directories left empty once the files moved.
fn prune(dir: &Path) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)?.flatten() {
        let path = entry.path();
        if path.is_dir() {
            prune(&path)?;
            let _ = std::fs::remove_dir(&path);
        }
    }
    Ok(())
}

/// Index the protobuf output by package.
///
/// `buf` writes one directory per package holding a message file and a service
/// file, both of which are included into the same module — splitting them would
/// put a service in a different module from the types it names.
fn protobuf(base: &Path) -> std::io::Result<Vec<ProtoPackage>> {
    let mut files = vec![];
    collect(base, ".rs", &mut files)?;
    let mut by_dir: std::collections::BTreeMap<PathBuf, Vec<String>> = Default::default();
    for f in files {
        let dir = f.parent().expect("file has a parent").to_path_buf();
        let rel = f.strip_prefix(base.parent().expect("generated has a parent"));
        let rel = rel.unwrap_or(&f).to_string_lossy().replace('\\', "/");
        by_dir
            .entry(dir)
            .or_default()
            .push(format!("generated/{rel}"));
    }
    let mut out = vec![];
    for (dir, mut includes) in by_dir {
        // The service file must follow the message file: it names those types.
        includes.sort();
        let package = package_of(dir.strip_prefix(base).unwrap_or(&dir));
        out.push(ProtoPackage { package, includes });
    }
    Ok(out)
}
