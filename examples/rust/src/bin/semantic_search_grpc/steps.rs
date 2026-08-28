//! The five steps, each announced before it runs.

use telividb_client::{Client, NewCollection};
use telividb_examples::corpus;

/// How long to wait for the server to load its model and bind.
const STARTUP_ATTEMPTS: usize = 600;

/// Connect once the server is accepting, or give up with a clear message.
pub(crate) async fn connect(addr: std::net::SocketAddr) -> Client {
    for _ in 0..STARTUP_ATTEMPTS {
        // The port accepting is necessary but not sufficient: the server binds
        // after loading its model, and a connect can still race the router.
        if std::net::TcpStream::connect(addr).is_ok()
            && let Ok(client) = Client::connect(format!("http://{addr}")).await
        {
            return client;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    eprintln!("the server did not start on {addr}");
    std::process::exit(1);
}

/// Step 2 — declare the collection and the field its points will carry.
///
/// An existing collection is reused rather than treated as a failure, so
/// running the example twice works.
pub(crate) async fn create_collection(client: &mut Client, id: &str, field: &str, dim: usize) {
    println!("[2/5] creating collection {id:?} with field {field:?} ({dim} dimensions, cosine)");

    let spec = NewCollection::new(id, descriptor_set()).text_field(field, dim);
    match client.create_collection(spec).await {
        Ok(name) => println!("      created {name:?}.\n"),
        Err(telividb_client::Error::AlreadyExists { .. }) => {
            println!("      already exists — reusing it.\n");
        }
        Err(e) => {
            eprintln!("      could not create the collection: {e}");
            std::process::exit(1);
        }
    }
}

/// Step 3 — add the corpus as text. Returns how many points the collection now
/// holds.
pub(crate) async fn add_documents(client: &mut Client, id: &str, field: &str) -> usize {
    println!("[3/5] adding {} documents as text", corpus::DOCUMENTS.len());
    println!("      (the server embeds them; this process holds no model)");
    let mut docs = client.collection(id);

    let mut added = 0usize;
    for (i, text) in corpus::DOCUMENTS.iter().enumerate() {
        let point_id = format!("doc-{i}");
        match docs.add_text(&point_id, field, text).await {
            Ok(_) => added += 1,
            // Present from an earlier run. Not an error: this example is meant
            // to be re-runnable against the same data directory.
            Err(telividb_client::Error::AlreadyExists { .. }) => {}
            Err(e) => {
                eprintln!("      could not add {point_id}: {e}");
                std::process::exit(1);
            }
        }
    }

    let total = docs.list().await.map(|p| p.len()).unwrap_or(added);
    println!("      added {added}, collection now holds {total}.\n");
    total
}

/// Step 4 — ask the sample questions.
pub(crate) async fn run_queries(client: &mut Client, id: &str, field: &str) {
    println!("[4/5] searching by text\n");
    let mut docs = client.collection(id);

    for question in corpus::QUERIES {
        let found = match docs.search_text(field, question, 3).await {
            Ok(found) => found,
            Err(e) => {
                eprintln!("  search failed: {e}");
                continue;
            }
        };

        println!("? {question}");
        for (rank, hit) in found.hits().iter().enumerate() {
            println!(
                "  {}. {:.4}  {}",
                rank + 1,
                hit.score,
                hit.text.as_deref().unwrap_or(&hit.name)
            );
        }
        // Reported rather than assumed: a partial answer must not read as a
        // complete one (rules 27 and 49).
        if !found.is_complete() {
            println!("  (incomplete — locked: {:?})", found.locked_vaults());
        }
        println!();
    }
}

/// A real compiled descriptor set.
///
/// This workspace's own. Not the schema a production collection would carry,
/// but genuinely a `FileDescriptorSet` — which is what the server refuses to
/// create a collection without, since it never parses `.proto` itself.
fn descriptor_set() -> Vec<u8> {
    telividb_buffers::protobuf::FILE_DESCRIPTOR_SET.to_vec()
}
