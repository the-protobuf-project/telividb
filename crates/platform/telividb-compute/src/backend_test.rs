use super::*;

#[test]
fn a_backend_initialises_and_names_itself() {
    let backend = Backend::best().expect("the host backend is always available");
    assert!(!backend.name().is_empty(), "ggml should name the backend");
    println!("opened: {} ({:?})", backend.name(), backend.device().kind());
}

#[test]
fn the_host_backend_is_always_reachable() {
    // Every fallback depends on it.
    let backend = Backend::of(DeviceKind::Cpu).expect("cpu backend");
    assert_eq!(backend.device().kind(), DeviceKind::Cpu);
}
