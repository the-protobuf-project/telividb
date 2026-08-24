//! Loading a BEIR dataset from its JSONL and TSV files.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A corpus document, as BEIR distributes it.
pub struct Document {
    /// The dataset's own id, which qrels reference.
    pub id: String,
    /// Title and body, joined the way BEIR's own evaluation does.
    pub text: String,
}

/// One query, with the documents judged relevant to it.
pub struct Query {
    /// The dataset's query id.
    pub id: String,
    /// The question text.
    pub text: String,
    /// Corpus ids judged relevant, with their graded scores.
    ///
    /// Graded rather than boolean because nDCG weights a highly relevant
    /// document above a marginally relevant one — collapsing the grades would
    /// silently change what is being measured.
    pub relevant: HashMap<String, u32>,
}

/// Where `examples/datasets/download.sh` puts its files.
pub fn datasets_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(|p| p.join("datasets"))
        .unwrap_or_else(|| PathBuf::from("datasets"))
}

/// Load a dataset's corpus and its judged test queries.
///
/// Returns `Err` with the command to run rather than panicking: a missing
/// download is the likeliest way a first run fails.
pub fn load(name: &str) -> Result<(Vec<Document>, Vec<Query>), String> {
    let dir = datasets_dir().join(name);
    if !dir.join("corpus.jsonl").exists() {
        return Err(format!(
            "no {name} dataset in {}.\n\nFetch it with:\n    {}/download.sh {name}",
            dir.display(),
            datasets_dir().display()
        ));
    }

    let documents = load_corpus(&dir.join("corpus.jsonl"))?;
    let qrels = load_qrels(&dir.join("qrels").join("test.tsv"))?;
    let queries = load_queries(&dir.join("queries.jsonl"), &qrels)?;
    Ok((documents, queries))
}

/// Read the corpus, joining each document's title to its body.
fn load_corpus(path: &Path) -> Result<Vec<Document>, String> {
    let raw = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;

    raw.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let value: serde_json::Value =
                serde_json::from_str(line).map_err(|e| format!("corpus line: {e}"))?;
            let title = value["title"].as_str().unwrap_or("");
            let body = value["text"].as_str().unwrap_or("");

            Ok(Document {
                id: value["_id"].as_str().unwrap_or_default().to_owned(),
                // Title and body joined with a space — what BEIR's reference
                // evaluation embeds. Embedding the body alone measures a
                // different system and would not be comparable.
                text: format!("{title} {body}").trim().to_owned(),
            })
        })
        .collect()
}

/// Read `qrels/test.tsv`: `query-id \t corpus-id \t score`, with a header.
fn load_qrels(path: &Path) -> Result<HashMap<String, HashMap<String, u32>>, String> {
    let raw = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;

    let mut qrels: HashMap<String, HashMap<String, u32>> = HashMap::new();
    for line in raw.lines().skip(1) {
        let mut parts = line.split('\t');
        let (Some(query), Some(document), Some(score)) = (parts.next(), parts.next(), parts.next())
        else {
            continue;
        };
        let score: u32 = score.trim().parse().unwrap_or(0);
        // A zero grade means "judged, and not relevant" — keeping it would
        // count a known-irrelevant document as a hit.
        if score > 0 {
            qrels
                .entry(query.to_owned())
                .or_default()
                .insert(document.to_owned(), score);
        }
    }
    Ok(qrels)
}

/// Read `queries.jsonl`, keeping only those with judgements.
///
/// BEIR ships every split's queries in one file, so without this the run would
/// score train queries against test qrels and report near-zero.
fn load_queries(
    path: &Path,
    qrels: &HashMap<String, HashMap<String, u32>>,
) -> Result<Vec<Query>, String> {
    let raw = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;

    let mut queries: Vec<Query> = raw
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let value: serde_json::Value = serde_json::from_str(line).ok()?;
            let id = value["_id"].as_str()?.to_owned();
            let relevant = qrels.get(&id)?.clone();
            Some(Query {
                text: value["text"].as_str().unwrap_or_default().to_owned(),
                id,
                relevant,
            })
        })
        .collect();

    // Sorted so two runs report the same numbers in the same order, which is
    // what makes a diff between them meaningful.
    queries.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(queries)
}
