use super::*;

#[test]
fn default_config_is_quiet_and_local() {
    let c = TelemetryConfig::default();
    assert_eq!(c.filter, "info");
    assert!(!c.json);
    assert!(c.prometheus.is_none(), "must not open a port unasked");
    assert_eq!(
        c.recall_sample_rate, 0.0,
        "recall sampling costs a full scan"
    );
}

#[test]
fn recall_sampling_is_off_when_rate_is_zero() {
    let t = Telemetry {
        config: TelemetryConfig::default(),
    };
    for draw in [0.0, 0.5, 0.999] {
        assert!(!t.should_sample_recall(draw));
    }
}

#[test]
fn recall_sampling_respects_the_rate() {
    let t = Telemetry {
        config: TelemetryConfig {
            recall_sample_rate: 0.01,
            ..Default::default()
        },
    };
    assert!(t.should_sample_recall(0.005), "below rate should sample");
    assert!(!t.should_sample_recall(0.01), "at rate should not");
    assert!(!t.should_sample_recall(0.9), "above rate should not");
}

#[test]
fn a_bad_filter_directive_is_rejected() {
    let err = Telemetry::install(TelemetryConfig {
        filter: "this is not=a=valid=filter".to_owned(),
        ..Default::default()
    });
    assert!(matches!(err, Err(TelemetryError::Filter(_))));
}
