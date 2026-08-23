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
    assert_eq!(c.service, "episteme");
    assert!(!c.version.is_empty(), "a collector groups by version");
}

#[test]
fn development_is_the_default_environment() {
    // Verbosity is the environment in this stack, not a log level. Defaulting
    // to development keeps a local run chatty without anyone configuring it.
    let c = TelemetryConfig::default();
    assert_eq!(c.environment.to_string().to_lowercase(), "development");
}

#[test]
fn recall_sampling_is_off_when_the_rate_is_zero() {
    for draw in [0.0, 0.5, 0.999] {
        assert!(!should_sample(0.0, draw));
    }
}

#[test]
fn recall_sampling_respects_the_rate() {
    assert!(should_sample(0.01, 0.005), "below the rate should sample");
    assert!(!should_sample(0.01, 0.01), "at the rate should not");
    assert!(!should_sample(0.01, 0.9), "above the rate should not");
}
