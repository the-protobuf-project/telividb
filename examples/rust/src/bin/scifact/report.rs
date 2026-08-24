//! Printing the result, and saying what it means.

use crate::run::Outcome;

/// Print the measured scores against the correctness floor.
pub fn print(outcome: &Outcome, floor: f64) {
    let report = &outcome.report;

    println!("---- scifact ----");
    println!("  device        : {}", outcome.device);
    println!("  dimensions    : {}", outcome.dim);
    println!("  queries       : {}", report.queries);
    println!();
    println!("  nDCG@10       : {:.4}", report.ndcg_at_10);
    println!("  Recall@10     : {:.4}", report.recall_at_10);
    println!("  Recall@100    : {:.4}", report.recall_at_100);
    println!();
    println!(
        "  corpus embed  : {:.1}s ({:.0} docs/s)",
        outcome.embed_time.as_secs_f64(),
        5183.0 / outcome.embed_time.as_secs_f64().max(1e-9),
    );
    println!(
        "  query + search: {:.1}s ({:.1}ms per query)",
        outcome.search_time.as_secs_f64(),
        outcome.search_time.as_secs_f64() * 1000.0 / (report.queries.max(1) as f64),
    );
    println!();

    if report.ndcg_at_10 >= floor {
        println!(
            "  PASS — nDCG@10 {:.4} is at or above the {floor:.2} floor.",
            report.ndcg_at_10
        );
        println!();
        println!("  The floor is a correctness check, not a quality target. It is set");
        println!("  well below what a healthy nomic-embed-text-v1.5 scores on SciFact,");
        println!("  because what it exists to catch is a broken encoder — a wrong");
        println!("  tokenizer, pooling mode or rotation convention — which lands far");
        println!("  lower and is otherwise invisible. Compare the number above against");
        println!("  the MTEB leaderboard entry for this model to judge quality.");
    } else {
        println!(
            "  FAIL — nDCG@10 {:.4} is below the {floor:.2} floor.",
            report.ndcg_at_10
        );
        println!();
        println!("  Something in the pipeline is wrong rather than merely mediocre.");
        println!("  Where to look, in order:");
        println!("    - the tokenizer: check for [UNK] on ordinary words");
        println!("    - pooling: mean versus CLS, read from the GGUF");
        println!("    - the task prefix: documents and queries take different ones");
        println!("    - normalisation: vectors must be unit length for Dot to be cosine");
        println!();
        println!("  Recall@100 tells the two apart: healthy recall with poor nDCG is a");
        println!("  ranking problem, and both low is an encoding problem.");
    }
}
