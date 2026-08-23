use super::*;

fn parse_args(args: &[&str]) -> Result<Option<ServerConfig>, String> {
    parse(args.iter().map(|s| (*s).to_owned()))
}

#[test]
fn no_arguments_uses_defaults() {
    let config = parse_args(&[]).unwrap().unwrap();
    assert_eq!(config.addr.port(), 7700);
    assert!(config.metrics_addr.is_none());
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
fn an_invalid_log_format_is_rejected() {
    let err = parse_args(&["--log-format", "yaml"]).unwrap_err();
    assert!(err.contains("text"), "{err}");
}

#[test]
fn help_returns_no_config() {
    assert!(parse_args(&["--help"]).unwrap().is_none());
    assert!(parse_args(&["-h"]).unwrap().is_none());
}

#[test]
fn metrics_stay_opt_in() {
    let config = parse_args(&["--metrics", "127.0.0.1:9100"])
        .unwrap()
        .unwrap();
    assert_eq!(config.metrics_addr.unwrap().port(), 9100);
}
