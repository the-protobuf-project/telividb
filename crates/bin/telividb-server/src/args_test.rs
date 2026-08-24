use super::*;

fn parse_args(args: &[&str]) -> Result<Option<ServerConfig>, String> {
    parse(args.iter().map(|s| (*s).to_owned()))
}

#[test]
fn no_arguments_uses_defaults() {
    let config = parse_args(&[]).unwrap().unwrap();
    assert_eq!(config.addr.port(), 7700);
    assert!(config.otlp_addr.is_none());
}

#[test]
fn addr_is_honoured() {
    let config = parse_args(&["--addr", "0.0.0.0:9000"]).unwrap().unwrap();
    assert_eq!(config.addr.port(), 9000);
}

#[test]
fn a_bare_separator_is_ignored() {
    // `cargo serve -- --addr ...` passes a stray `--` through. It used to be
    // treated as an unknown flag, which desynchronised the flag/value pairing
    // and left the server listening on the default port instead of the one
    // asked for — with only a warning.
    let config = parse_args(&["--", "--addr", "127.0.0.1:7820"])
        .unwrap()
        .unwrap();
    assert_eq!(
        config.addr.port(),
        7820,
        "the address must survive a stray --"
    );
}

#[test]
fn an_unknown_flag_is_fatal() {
    // Binding a different address than requested is worse than refusing to
    // start: the mistake stays invisible until something cannot reach it.
    let err = parse_args(&["--port", "9000"]).unwrap_err();
    assert!(err.contains("unknown flag"), "{err}");
}

#[test]
fn a_flag_without_a_value_is_rejected() {
    assert!(
        parse_args(&["--addr"])
            .unwrap_err()
            .contains("needs a value")
    );
}

#[test]
fn a_malformed_address_is_rejected() {
    let err = parse_args(&["--addr", "not-an-address"]).unwrap_err();
    assert!(err.contains("--addr"), "{err}");
}

#[test]
fn the_environment_is_validated() {
    let config = parse_args(&["--environment", "production"])
        .unwrap()
        .unwrap();
    assert!(matches!(config.environment, Environment::Production));

    let err = parse_args(&["--environment", "prod"]).unwrap_err();
    assert!(err.contains("development"), "{err}");
}

#[test]
fn jetson_is_a_recognised_environment() {
    // The telemetry stack targets it directly, and it is a real deployment
    // target for this database — CUDA on aarch64.
    let config = parse_args(&["--environment", "jetson"]).unwrap().unwrap();
    assert!(matches!(config.environment, Environment::Jetson));
}

#[test]
fn help_returns_no_config() {
    assert!(parse_args(&["--help"]).unwrap().is_none());
    assert!(parse_args(&["-h"]).unwrap().is_none());
}

#[test]
fn otlp_export_stays_opt_in() {
    // A database should not send telemetry off the machine because nobody
    // said not to.
    assert!(parse_args(&[]).unwrap().unwrap().otlp_addr.is_none());
    let config = parse_args(&["--otlp", "127.0.0.1:4317"]).unwrap().unwrap();
    assert_eq!(config.otlp_addr.unwrap().port(), 4317);
}

#[test]
fn mcap_recording_stays_opt_in() {
    assert!(parse_args(&[]).unwrap().unwrap().mcap_path.is_none());
    let config = parse_args(&["--mcap", "/tmp/run.mcap"]).unwrap().unwrap();
    assert_eq!(config.mcap_path.unwrap().to_str().unwrap(), "/tmp/run.mcap");
}

#[test]
fn data_dir_defaults_to_a_relative_path_and_is_overridable() {
    assert_eq!(
        parse_args(&[]).unwrap().unwrap().data_dir.to_str().unwrap(),
        "./data"
    );
    let config = parse_args(&["--data-dir", "/var/lib/telividb"])
        .unwrap()
        .unwrap();
    assert_eq!(config.data_dir.to_str().unwrap(), "/var/lib/telividb");
}
