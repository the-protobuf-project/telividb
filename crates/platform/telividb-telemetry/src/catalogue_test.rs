use super::*;

#[test]
fn every_counter_says_so_in_its_name() {
    // Prometheus convention, and the reason the kind is worth carrying: a
    // counter that does not end `_total` reads as a gauge on every dashboard
    // that follows the convention.
    for (name, kind, _) in ALL {
        if matches!(kind, MetricType::Counter) {
            assert!(
                name.ends_with("_total"),
                "{name} is a counter but does not end _total"
            );
        }
    }
}

#[test]
fn no_gauge_is_named_like_a_counter() {
    for (name, kind, _) in ALL {
        if matches!(kind, MetricType::Gauge) {
            assert!(
                !name.ends_with("_total"),
                "{name} is a gauge named like a counter"
            );
        }
    }
}

#[test]
fn durations_are_histograms() {
    // A duration recorded as a counter or a gauge cannot yield a quantile,
    // which is the only reading of a latency that means anything.
    for (name, kind, _) in ALL {
        if name.ends_with("_duration_seconds") {
            assert!(
                matches!(kind, MetricType::Histogram),
                "{name} is a duration but not a histogram"
            );
        }
    }
}

#[test]
fn the_live_gauges_are_gauges() {
    // These were all registered as histograms before the kind was carried
    // here, which mislabelled a third of the catalogue at the collector.
    for name in [SEGMENTS_LIVE, ROWS_LIVE, ROWS_TOMBSTONED] {
        let (_, kind, _) = ALL.iter().find(|(n, _, _)| *n == name).expect("catalogued");
        assert!(matches!(kind, MetricType::Gauge), "{name} is not a gauge");
    }
}
