use super::*;
use telividb_core::Edge;

fn name(s: &str) -> ResourceName {
    ResourceName::parse(s).unwrap()
}

/// `seed -> a -> b -> c`, all `NEXT` edges, plus a `MENTIONS` edge from `seed`
/// straight to `c` so the edge-type filter has something to exclude.
fn chain() -> Graph {
    let mut graph = Graph::new();
    graph.insert_edge(Edge::new(name("seed"), name("a"), "NEXT", 1.0));
    graph.insert_edge(Edge::new(name("a"), name("b"), "NEXT", 1.0));
    graph.insert_edge(Edge::new(name("b"), name("c"), "NEXT", 1.0));
    graph.insert_edge(Edge::new(name("seed"), name("c"), "MENTIONS", 1.0));
    graph
}

fn sorted(mut names: Vec<ResourceName>) -> Vec<ResourceName> {
    names.sort();
    names
}

#[test]
fn zero_hops_reaches_nothing() {
    let graph = chain();
    assert!(graph.k_hop(&name("seed"), 0, None, None).is_empty());
}

#[test]
fn one_hop_reaches_only_direct_neighbours() {
    let graph = chain();
    let reached = sorted(graph.k_hop(&name("seed"), 1, None, None));
    assert_eq!(reached, vec![name("a"), name("c")]);
}

#[test]
fn multi_hop_reaches_the_whole_chain() {
    let graph = chain();
    let reached = sorted(graph.k_hop(&name("seed"), 3, None, None));
    assert_eq!(reached, vec![name("a"), name("b"), name("c")]);
}

#[test]
fn edge_type_filter_excludes_the_other_relationship() {
    let graph = chain();
    let reached = graph.k_hop(&name("seed"), 3, Some("NEXT"), None);
    assert_eq!(
        sorted(reached),
        vec![name("a"), name("b"), name("c")],
        "NEXT alone still reaches c via a -> b -> c"
    );

    let mentions_only = graph.k_hop(&name("seed"), 1, Some("MENTIONS"), None);
    assert_eq!(mentions_only, vec![name("c")]);
}

#[test]
fn a_disallowed_node_is_excluded_and_not_expanded_past() {
    let graph = chain();
    let allowed = |n: &ResourceName| n != &name("a");
    let reached = graph.k_hop(&name("seed"), 3, None, Some(&allowed));
    assert_eq!(
        reached,
        vec![name("c")],
        "a is hidden, and b/nothing beyond a-that-was-hidden is reachable through it"
    );
}

#[test]
fn an_unknown_seed_reaches_nothing() {
    let graph = chain();
    assert!(graph.k_hop(&name("missing"), 5, None, None).is_empty());
}
