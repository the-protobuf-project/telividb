use super::*;

#[test]
fn a_device_is_always_selected() {
    // The fallback is the point: a build with no GPU backend compiled in must
    // still resolve to something searchable rather than failing.
    let device = best_device();
    assert!(matches!(device_name(&device), "metal" | "cuda" | "cpu"));
}

#[test]
fn cpu_names_itself() {
    assert_eq!(device_name(&Device::Cpu), "cpu");
}

#[test]
#[cfg(target_os = "macos")]
fn metal_is_selected_on_macos() {
    // Guards the failure no correctness test can catch: a GPU index that has
    // quietly fallen back to CPU passes everything else while delivering none
    // of the speed. Metal is compiled in automatically on macOS, so if this
    // fails here the backend did not initialise and the fallback is hiding it.
    assert_eq!(
        device_name(&best_device()),
        "metal",
        "metal is compiled in on macOS but the device did not initialise"
    );
}
