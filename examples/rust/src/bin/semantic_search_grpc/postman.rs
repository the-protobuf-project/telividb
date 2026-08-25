//! What to paste into Postman once the example is seeded.
//!
//! Printed rather than documented elsewhere because the details that matter —
//! the address, the collection name, the field — are decided at runtime, and a
//! README repeating them would be the copy that goes stale.

/// Print everything needed to query the seeded data from another tool.
pub(crate) fn print_notes(addr: &str, collection: &str, field: &str, points: usize) {
    println!("[5/5] ready for Postman\n");
    println!("  address        : {addr}   (gRPC, plaintext — no TLS)");
    println!("  reflection     : enabled, so Postman can list the services itself");
    println!("  gRPC-web       : enabled on the same port");
    println!("  seeded         : {points} points in {collection:?}\n");

    println!("  In Postman: New > gRPC, enter {addr}, and choose");
    println!("  \"Using server reflection\". Then pick:\n");
    println!("      telividb.point.v1.Points / SearchPoints\n");
    println!("  Message body — searching by text, the server embeds it:\n");
    println!("{}", search_body(collection, field));

    println!("\n  The same call with grpcurl:\n");
    println!(
        "      grpcurl -plaintext -d '{}' \\\n        {addr} telividb.point.v1.Points/SearchPoints",
        compact(&search_body(collection, field))
    );

    println!("\n  To add a document (also plain text):\n");
    println!(
        "      grpcurl -plaintext -d '{}' \\\n        {addr} telividb.point.v1.Points/CreatePoint",
        compact(&create_body(collection, field))
    );

    println!("\n  Note `query_text` rather than `query`: exactly one of the two");
    println!("  is set. `query` takes a base64 `bytes` field of raw little-endian");
    println!("  f32, which is awkward to type by hand — `query_text` lets the");
    println!("  server encode it with the model the field is bound to.");
}

/// A `SearchPoints` request body.
fn search_body(collection: &str, field: &str) -> String {
    format!(
        r#"      {{
        "parent": "collections/{collection}",
        "field_id": "{field}",
        "query_text": "Where did the cat sit?",
        "page_size": 3
      }}"#
    )
}

/// A `CreatePoint` request body.
fn create_body(collection: &str, field: &str) -> String {
    format!(
        r#"      {{
        "parent": "collections/{collection}",
        "point_id": "doc-100",
        "point": {{
          "vectors": [{{ "field_id": "{field}", "text": "A new sentence to index." }}],
          "content_ref": {{ "inline_text": "A new sentence to index." }}
        }}
      }}"#
    )
}

/// Squash a pretty-printed body onto one line for a shell command.
fn compact(body: &str) -> String {
    body.split_whitespace().collect::<Vec<_>>().join(" ")
}
