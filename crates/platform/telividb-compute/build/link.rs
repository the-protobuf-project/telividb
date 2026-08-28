//! What to link the vendored ggml build against.
//!
//! Split from `build.rs` because it is a different job: everything there
//! produces artifacts, everything here describes them to rustc. The two also
//! fail differently — a mistake there stops the build with a compiler error,
//! a mistake here surfaces as an undefined symbol while linking some other
//! crate entirely.

use std::path::Path;

/// Tell Cargo what to link.
///
/// The libraries are **discovered rather than listed**. CMake decides which
/// backends to build from the platform and the feature flags — it enabled a
/// BLAS backend on macOS that no feature asked for — so a hardcoded list goes
/// stale the moment ggml adds one, and fails at link time with an undefined
/// symbol that names a backend rather than the real problem.
pub fn link(install: &Path) {
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
    }

    // ggml is C++, so the C++ runtime has to be linked explicitly — rustc
    // drives the final link and, unlike a C++ compiler driver, adds nothing.
    // Without it every `std::` symbol in the vendored sources is undefined,
    // and the failure lands at link time in whichever crate happens to be
    // linked first rather than anywhere near this file.
    //
    // The implementation differs by platform and cannot be guessed: Apple's
    // toolchain ships libc++, and the GNU toolchain every Linux runner uses
    // ships libstdc++. Naming the wrong one fails exactly as loudly as naming
    // none.
    println!(
        "cargo:rustc-link-lib={}",
        if cfg!(target_os = "macos") {
            "c++"
        } else {
            "stdc++"
        }
    );
}
