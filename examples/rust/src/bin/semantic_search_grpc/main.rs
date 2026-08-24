//! The same semantic search, over gRPC, with no model on the client.
//!
//! ```text
//! examples/models/download.sh
//! cargo run --release -p telividb-examples --bin semantic_search_grpc
//! ```
//!
//! The companion to `semantic_search`, which does everything in-process. This
//! one starts a real server, then talks to it through the SDK exactly as a
//! separate process would — so the client here holds no model, no index and no
//! storage. It sends sentences and gets ranked sentences back.
//!
//! That split is the point. A field's vectors are bound to one model identity
//! (rule 12), and keeping inference on the server is what makes that hold no
//! matter how many clients write to it.
//!
//! The server is started in-process only so the example is one command. In a
//! deployment it is a separate `telividb-server --model <path>`, and the
//! client code below is unchanged.

use telividb_client::Client;
use telividb_examples::{corpus, model};
use telividb_server::{ServerConfig, serve};

/// Field the corpus is stored under.
const FIELD: &str = "text";

/// How long to wait for the server to load its model and bind.
const STARTUP_ATTEMPTS: usize = 600;

#[tokio::main]
async fn main() {
    let model_path = match model::default_text_model() {
        Ok(path) => path,
        Err(explanation) => {
            eprintln!("{explanation}");
            std::process::exit(1);
        }
    };

    let data = tempfile::tempdir().expect("temp dir");
    let addr = {
        // Bind to find a free port, then release it for the server to take.
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("port");
        let addr = listener.local_addr().expect("addr");
        drop(listener);
        addr
    };

    println!("starting a server with {}...", model_path.display());
    let data_dir = data.path().to_path_buf();
    tokio::spawn(async move {
        let outcome = serve(ServerConfig {
            environment: telividb_telemetry::Environment::Production,
            data_dir,
            model_path: Some(model_path),
            model_name: "nomic-embed-text-v1.5".to_owned(),
            ..ServerConfig::at(addr)
        })
        .await;
        if let Err(e) = outcome {
            eprintln!("server failed: {e}");
        }
    });

    let client = connect(addr).await;
    println!("connected to {addr}\n");

    // ---- Everything below is ordinary SDK use. ----
    let mut docs = client.collection("documents");

    let entries: Vec<(String, String)> = corpus::DOCUMENTS
        .iter()
        .enumerate()
        .map(|(i, text)| (format!("doc-{i}"), (*text).to_owned()))
        .collect();

    // Text in. The server embeds it; this process has no model.
    docs.add_texts(FIELD, &entries)
        .await
        .expect("store the corpus");
    println!("stored {} documents.\n", entries.len());

    for question in corpus::QUERIES {
        let found = docs.search_text(FIELD, question, 3).await.expect("search");

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

    println!("The client never loaded a model. Every vector here was computed");
    println!("by the server, which is what keeps one field bound to one model");
    println!("however many clients write to it.");
}

/// Connect once the server is accepting, or give up with a clear message.
async fn connect(addr: std::net::SocketAddr) -> Client {
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
