use super::*;

/// A graph with several levels and uneven neighbour counts.
fn sample() -> Graph {
    let mut g = Graph::new();
    g.push_node(0);
    g.push_node(2);
    g.push_node(1);
    g.push_node(0);

    g.set_neighbours(0, 0, vec![1, 2, 3]);
    g.set_neighbours(1, 0, vec![0, 2]);
    g.set_neighbours(1, 1, vec![2]);
    g.set_neighbours(1, 2, vec![]);
    g.set_neighbours(2, 0, vec![0, 1, 3]);
    g.set_neighbours(2, 1, vec![1]);
    g.set_neighbours(3, 0, vec![0]);
    g
}

fn assert_same(a: &Graph, b: &Graph) {
    assert_eq!(a.len(), b.len(), "node count");
    assert_eq!(a.max_level(), b.max_level(), "max level");
    assert_eq!(a.entry(), b.entry(), "entry point");
    assert_eq!(a.edge_count(), b.edge_count(), "edge count");
    for node in 0..a.len() as u32 {
        assert_eq!(a.level_of(node), b.level_of(node), "level of {node}");
        for layer in 0..=a.level_of(node) {
            assert_eq!(
                a.neighbours(node, layer),
                b.neighbours(node, layer),
                "neighbours of {node} at {layer}"
            );
        }
    }
}

#[test]
fn round_trips_exactly() {
    let original = sample();
    assert_same(&original, &decode(&encode(&original)).unwrap());
}

#[test]
fn an_empty_graph_round_trips() {
    let empty = Graph::new();
    let back = decode(&encode(&empty)).unwrap();
    assert!(back.is_empty());
    assert_eq!(back.entry(), None);
}

#[test]
fn a_single_node_round_trips() {
    let mut g = Graph::new();
    g.push_node(0);
    let back = decode(&encode(&g)).unwrap();
    assert_eq!(back.len(), 1);
    assert_eq!(back.entry(), g.entry());
}

#[test]
fn empty_neighbour_lists_survive() {
    // Node 1 has no neighbours at its top layer. A format that skipped empty
    // lists would shift every subsequent offset.
    let back = decode(&encode(&sample())).unwrap();
    assert!(back.neighbours(1, 2).is_empty());
    assert_eq!(back.neighbours(1, 1), &[2]);
}

#[test]
fn a_foreign_file_is_rejected() {
    let mut bytes = encode(&sample());
    bytes[0..4].copy_from_slice(b"PARQ");
    assert!(matches!(decode(&bytes), Err(Error::MalformedIndex { .. })));
}

#[test]
fn a_newer_version_is_refused_not_guessed() {
    let mut bytes = encode(&sample());
    bytes[4..6].copy_from_slice(&(GRAPH_VERSION + 1).to_le_bytes());
    assert!(matches!(decode(&bytes), Err(Error::MalformedIndex { .. })));
}

#[test]
fn truncation_at_any_point_is_an_error_never_a_panic() {
    // A graph file is untrusted input once archives arrive from elsewhere.
    let bytes = encode(&sample());
    for cut in 0..bytes.len() {
        let result = decode(&bytes[..cut]);
        assert!(result.is_err(), "truncation at {cut} was accepted");
    }
}

#[test]
fn a_lying_length_field_is_an_error_never_an_overrun() {
    let mut bytes = encode(&sample());
    // Node count sits at offset 8. Claim far more nodes than are present.
    bytes[8..16].copy_from_slice(&u64::MAX.to_le_bytes());
    assert!(decode(&bytes).is_err());
}

#[test]
fn encoding_is_deterministic() {
    // Byte-identical output over unchanged input is what makes an archive
    // round-trip checkable.
    assert_eq!(encode(&sample()), encode(&sample()));
}
