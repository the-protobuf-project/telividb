//! Semantic search over gRPC, step by step — then left running for Postman.
//!
//! ```text
//! examples/models/download.sh
//! cargo run --release -p telividb-examples --bin semantic_search_grpc
//! ```
//!
//! Walks the flow a real caller follows, in order and announced:
//!
//! 1. start a server holding the embedding model
//! 2. **create a collection**, declaring the field its points will carry
//! 3. add documents as *text* — the server embeds them
//! 4. ask questions, also as text
//! 5. stay running, so the same data can be queried from Postman or grpcurl
//!
//! Step 2 is not a formality. A field's width, metric and model are bound at
//! declaration; left to be inferred from the first write they become whatever
//! that writer happened to send, and a later one either collides or is quietly
//! merged in. The server refuses a point whose collection does not exist.
//!
//! **Idempotent.** Data lives under `./data/example` and is reused, so running
//! this twice does not fail on what already exists.

mod postman;
mod steps;

use telividb_examples::model;
use telividb_server::{ServerConfig, serve};

/// Fixed rather than ephemeral, so the address in the Postman notes is the
/// address that is actually listening.
const ADDR: &str = "127.0.0.1:7700";

/// The collection this example seeds.
const COLLECTION: &str = "documents";

/// Field name, and the width nomic-embed-text-v1.5 produces.
const FIELD: &str = "text";
/// Vector width of the model this example loads.
const DIM: usize = 768;

#[tokio::main]
async fn main() {
    let model_path = match model::default_text_model() {
        Ok(path) => path,
        Err(explanation) => {
            eprintln!("{explanation}");
            std::process::exit(1);
        }
    };

    let addr = ADDR.parse().expect("literal is a valid address");
    let data_dir = std::path::PathBuf::from("./data/example");

    println!("[1/5] starting a server on {ADDR}");
    println!("      model    : {}", model_path.display());
    println!("      data dir : {}", data_dir.display());
    let serving_dir = data_dir.clone();
    tokio::spawn(async move {
        let outcome = serve(ServerConfig {
            environment: telividb_telemetry::Environment::Production,
            data_dir: serving_dir,
            model_path: Some(model_path),
            model_name: "nomic-embed-text-v1.5".to_owned(),
            ..ServerConfig::at(addr)
        })
        .await;
        if let Err(e) = outcome {
            eprintln!("server failed: {e}");
        }
    });

    let mut client = steps::connect(addr).await;
    println!("      connected.\n");

    steps::create_collection(&mut client, COLLECTION, FIELD, DIM).await;
    let stored = steps::add_documents(&mut client, COLLECTION, FIELD).await;
    steps::run_queries(&mut client, COLLECTION, FIELD).await;

    postman::print_notes(ADDR, COLLECTION, FIELD, stored);

    // Left running deliberately: the point of the fixed address above is that
    // the data seeded here can be queried from something else.
    println!("\nServing on {ADDR}. Press Ctrl-C to stop.");
    if let Err(e) = tokio::signal::ctrl_c().await {
        eprintln!("could not wait for Ctrl-C: {e}");
    }
    println!("stopped.");
}
