use super::*;

#[test]
fn the_host_is_always_reachable() {
    // Every fallback path depends on this, so it is worth asserting rather
    // than assuming.
    assert!(DeviceKind::compiled().contains(&DeviceKind::Cpu));
    assert_eq!(Device::cpu().kind(), DeviceKind::Cpu);
    assert!(!Device::cpu().is_accelerator());
}

#[test]
fn the_best_device_is_one_this_build_actually_has() {
    // `best` must never name a backend that was not compiled in — that would
    // fail at the first dispatch rather than at selection.
    let device = Device::best();
    assert!(DeviceKind::compiled().contains(&device.kind()));
}

#[test]
fn an_uncompiled_backend_is_refused_rather_than_silently_downgraded() {
    // A silent downgrade is the failure this whole enum exists to prevent: it
    // passes every correctness test while delivering none of the speed.
    for kind in [
        DeviceKind::Cuda,
        DeviceKind::Hip,
        DeviceKind::Vulkan,
        DeviceKind::Sycl,
    ] {
        if !DeviceKind::compiled().contains(&kind) {
            let refused = Device::of(kind);
            assert!(
                refused.is_err(),
                "{kind:?} should be refused when not built"
            );
        }
    }
}

#[test]
fn every_kind_has_a_distinct_label() {
    // The labels reach telemetry, where two kinds sharing one name would make
    // the series unjoinable.
    let all = [
        DeviceKind::Cpu,
        DeviceKind::Metal,
        DeviceKind::Cuda,
        DeviceKind::Hip,
        DeviceKind::Vulkan,
        DeviceKind::Sycl,
    ];
    let mut seen: Vec<&str> = all.iter().map(|k| k.as_str()).collect();
    seen.sort_unstable();
    let count = seen.len();
    seen.dedup();
    assert_eq!(seen.len(), count, "two device kinds share a label");
}

#[cfg(target_os = "macos")]
#[test]
fn metal_is_compiled_in_on_macos() {
    // Not behind a feature: it is the only GPU on this platform, and a default
    // that had to be opted into would leave it off for everyone.
    assert!(DeviceKind::compiled().contains(&DeviceKind::Metal));

    // What `best()` selects is only Metal when nothing overrides it. The
    // override is read rather than set here: a test that mutated the
    // environment would race every other test in the process, and the two
    // claims — "Metal is available" and "Metal is chosen by default" — are
    // separable anyway.
    if std::env::var_os("TELIVIDB_DEVICE").is_none() {
        assert_eq!(Device::best().kind(), DeviceKind::Metal);
    }
}

#[test]
fn every_device_name_round_trips() {
    // `parse` is the inverse of `as_str`, so the names an operator writes in
    // `TELIVIDB_DEVICE` are exactly the ones telemetry reports back. Tested as
    // a pure function, with no environment involved.
    for kind in DeviceKind::compiled() {
        assert_eq!(DeviceKind::parse(kind.as_str()), Some(*kind));
    }
    assert_eq!(DeviceKind::parse("  CPU  "), Some(DeviceKind::Cpu));
    assert_eq!(DeviceKind::parse("gpu"), None);
}
