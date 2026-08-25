//! How long an operation took, and how that time divided.
//!
//! Split from the main vocabulary because these are read *together*: a duration
//! alone says a query was slow, while the pair below says which half of it was
//! slow, and that is the difference between a number and a diagnosis.

/// Wall-clock duration of whatever the record covers, in seconds.
///
/// Carried on the log record as well as in a histogram: the histogram is
/// what a dashboard reads, and this is what someone reading one operation's
/// log line needs in order to see why it was slow.
pub const DURATION_SECONDS: &str = "telividb.duration_seconds";
/// How many queries one batched operation covered.
///
/// The denominator for [`SCORE_SECONDS`] and [`SELECT_SECONDS`], which are
/// reported for the batch as a whole: one matmul covers every query in it, so a
/// per-query figure would be an average the hardware never produced.
pub const QUERIES: &str = "telividb.queries";
/// Seconds spent on the device: query upload, the matmul, and the copy back.
///
/// On the log record beside [`SELECT_SECONDS`] rather than as a metric label,
/// per the rule that dimensions travel on the record: the pair is read together
/// when explaining one slow query, and a histogram alone cannot show that this
/// particular query spent its time on the wrong side.
pub const SCORE_SECONDS: &str = "telividb.score_seconds";
/// Seconds spent on the host applying the metric and selecting the best `k`.
pub const SELECT_SECONDS: &str = "telividb.select_seconds";
