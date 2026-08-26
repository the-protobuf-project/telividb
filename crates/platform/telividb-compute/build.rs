//! Builds vendored ggml and generates its Rust declarations.
//!
//! Two steps, both deliberate. CMake compiles ggml for whichever backends the
//! enabled features name, and `bindgen` reads ggml's own headers to produce the
//! raw declarations — so the bindings cannot drift from the C API the way a
//! hand-written set would.
//!
//! Backends are selected by Cargo feature rather than detected, because a build
//! that silently picks a different backend than the last one produces a binary
//! whose performance nobody can explain.

use std::path::{Path, PathBuf};

fn main() {
    let vendor = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("vendor/ggml");
    if !vendor.join("CMakeLists.txt").exists() {
        panic!(
            "ggml source is missing at {}.\n\
             It is a git submodule; run `git submodule update --init --recursive`.",
            vendor.display()
        );
    }

    println!("cargo:rerun-if-changed=build.rs");
    println!(
        "cargo:rerun-if-changed={}",
        vendor.join("include").display()
    );

    let install = build_ggml(&vendor);
    generate_bindings(&vendor);
    link(&install);
}

/// Compile ggml, returning where CMake installed it.
fn build_ggml(vendor: &PathBuf) -> PathBuf {
    let mut cfg = cmake::Config::new(vendor);
    cfg.define("GGML_BUILD_TESTS", "OFF")
        .define("GGML_BUILD_EXAMPLES", "OFF")
        // Static, so a deployed binary carries ggml rather than needing it
        // installed alongside.
        .define("BUILD_SHARED_LIBS", "OFF")
        .profile("Release");

    // **Instruction-set targeting, stated rather than inherited.**
    //
    // ggml's own default for `GGML_NATIVE` is ON, which compiles with
    // `-march=native`: fastest on the machine that built it, and liable to
    // fault with SIGILL on any older CPU. That is right for a local build and
    // wrong for a distributed binary, and the difference is invisible until
    // someone runs a Homebrew bottle on a machine without AVX-512.
    //
    // Set here so the choice is a decision with a name on it. `portable`
    // targets the architecture baseline instead — measurably slower on x86,
    // and free on aarch64 where NEON *is* the baseline.
    //
    // True per-machine dispatch (`GGML_CPU_ALL_VARIANTS`) is not reachable from
    // here: it requires `GGML_BACKEND_DL`, which needs shared libraries, and
    // this build is static so a deployed binary carries ggml rather than
    // needing it installed alongside. Revisit together, or not at all.
    cfg.define(
        "GGML_NATIVE",
        if cfg!(feature = "portable") {
            "OFF"
        } else {
            "ON"
        },
    );

    // Metal is on wherever it exists rather than behind a feature: it is the
    // only GPU on macOS, and `default-features` cannot be made conditional on
    // the target.
    cfg.define(
        "GGML_METAL",
        if cfg!(target_os = "macos") {
            "ON"
        } else {
            "OFF"
        },
    );
    cfg.define(
        "GGML_CUDA",
        if cfg!(feature = "cuda") { "ON" } else { "OFF" },
    );
    cfg.define("GGML_HIP", if cfg!(feature = "hip") { "ON" } else { "OFF" });
    cfg.define(
        "GGML_VULKAN",
        if cfg!(feature = "vulkan") {
            "ON"
        } else {
            "OFF"
        },
    );
    cfg.define(
        "GGML_SYCL",
        if cfg!(feature = "sycl") { "ON" } else { "OFF" },
    );

    cfg.build()
}

/// Turn ggml's headers into Rust declarations.
///
/// Only the headers the safe layer above actually wraps: the tensor API, the
/// backend abstraction and GGUF. Binding every per-backend header would expose
/// vendor-specific entry points that the abstraction exists to hide.
fn generate_bindings(vendor: &Path) {
    let include = vendor.join("include");
    let builder = bindgen::Builder::default()
        .header(include.join("ggml.h").to_string_lossy())
        .header(include.join("ggml-backend.h").to_string_lossy())
        // GGUF too: a corpus persists as one, and the format carries its own
        // magic and version, which is what rule 4 asks of an on-disk structure.
        .header(include.join("gguf.h").to_string_lossy())
        .clang_arg(format!("-I{}", include.display()))
        // Only ggml's own symbols; without this the bindings would carry every
        // libc declaration the headers transitively include.
        .allowlist_function("ggml_.*")
        .allowlist_function("gguf_.*")
        .allowlist_type("ggml_.*")
        .allowlist_type("gguf_.*")
        .allowlist_var("GGML_.*")
        .allowlist_var("GGUF_.*")
        .derive_debug(true)
        .generate_comments(false)
        .layout_tests(false)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()));

    let bindings = builder.generate().expect("ggml headers did not parse");
    let out = PathBuf::from(std::env::var("OUT_DIR").expect("cargo sets OUT_DIR"));
    bindings
        .write_to_file(out.join("ggml.rs"))
        .expect("could not write bindings");
}

/// Tell Cargo what to link.
///
/// The libraries are **discovered rather than listed**. CMake decides which
/// backends to build from the platform and the feature flags — it enabled a
/// BLAS backend on macOS that no feature asked for — so a hardcoded list goes
/// stale the moment ggml adds one, and fails at link time with an undefined
/// symbol that names a backend rather than the real problem.
fn link(install: &Path) {
    let lib_dir = install.join("lib");
    println!("cargo:rustc-link-search=native={}", lib_dir.display());

    let mut found = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&lib_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            // `libggml-cpu.a` -> `ggml-cpu`
            // `libggml-cpu.a` -> `ggml-cpu`
            if let Some(stem) = name
                .strip_prefix("lib")
                .and_then(|n| n.strip_suffix(".a"))
                .filter(|stem| stem.starts_with("ggml"))
            {
                found.push(stem.to_owned());
            }
        }
    }
    assert!(
        !found.is_empty(),
        "cmake produced no ggml static libraries in {}",
        lib_dir.display()
    );

    // Dependants before dependencies: every backend registers itself with
    // `ggml-base`, so base has to resolve last.
    found.sort_by_key(|name| match name.as_str() {
        "ggml" => 0,
        "ggml-base" => 2,
        _ => 1,
    });
    for lib in &found {
        println!("cargo:rustc-link-lib=static={lib}");
    }
    println!("cargo:warning=linking ggml backends: {}", found.join(", "));

    if cfg!(target_os = "macos") {
        // ggml's Metal backend is Objective-C, and its BLAS backend is
        // Accelerate.
        for framework in ["Metal", "MetalKit", "Foundation", "Accelerate"] {
            println!("cargo:rustc-link-lib=framework={framework}");
        }
        println!("cargo:rustc-link-lib=c++");
    }
}
