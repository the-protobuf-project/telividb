use super::*;

#[test]
fn defaults_keep_everything_local() {
    let c = TelemetryConfig::default();
    assert!(c.otlp.is_none(), "must not export off the machine unasked");
    assert!(c.mcap_path.is_none());
    assert_eq!(
        c.recall_sample_rate, 0.0,
        "recall sampling costs a full scan"
    );
}

#[test]
fn the_service_identifies_itself() {
    let c = TelemetryConfig::default();
    assert_eq!(c.service, "telividb");
    assert!(!c.version.is_empty(), "a collector groups by version");
}

#[test]
fn no_log_level_is_set_so_the_config_file_decides() {
    // A code-level override outranks `telemetry.toml`. Setting one by default
    // would make the file's `[logging] level` permanently dead.
    assert!(TelemetryConfig::default().log_level.is_none());
}

#[test]
fn an_ipv4_host_drops_only_the_port() {
    let addr: SocketAddr = "127.0.0.1:4317".parse().unwrap();
    assert_eq!(otlp_host(&addr), "127.0.0.1");
}

#[test]
fn an_ipv6_host_keeps_its_brackets() {
    // The stack rejoins host and port as `{host}:{port}`. Without the brackets
    // `[::1]:4317` becomes `::1:4317`, which is not a valid URI — and the
    // exporter fails somewhere far from here, or never connects at all.
    let addr: SocketAddr = "[::1]:4317".parse().unwrap();
    assert_eq!(otlp_host(&addr), "[::1]");
    assert_eq!(
        format!("{}:{}", otlp_host(&addr), addr.port()),
        "[::1]:4317"
    );
}

#[test]
fn a_full_ipv6_address_survives_intact() {
    let addr: SocketAddr = "[2001:db8::1]:4317".parse().unwrap();
    assert_eq!(otlp_host(&addr), "[2001:db8::1]");
}

#[test]
fn an_mcap_path_is_not_forced_through_display() {
    // A `String` field here is what let a non-UTF-8 path be rewritten with
    // U+FFFD, so the pipeline opened one file and reported another.
    let c = TelemetryConfig {
        mcap_path: Some(PathBuf::from("/var/log/telividb.mcap")),
        ..TelemetryConfig::default()
    };
    assert_eq!(
        c.mcap_path.as_deref(),
        Some(std::path::Path::new("/var/log/telividb.mcap"))
    );
}
