use super::*;
use telividb_core::Edge;

fn name(s: &str) -> ResourceName {
    ResourceName::parse(s).unwrap()
}

#[test]
fn a_new_graph_is_empty() {
    let graph = Graph::new();
    assert_eq!(graph.node_count(), 0);
    assert_eq!(graph.edge_count(), 0);
}

#[test]
fn inserting_an_edge_creates_both_endpoints() {
    let mut graph = Graph::new();
    graph.insert_edge(Edge::new(name("a/1"), name("b/1"), "MENTIONS", 1.0));
    assert_eq!(graph.node_count(), 2);
    assert_eq!(graph.edge_count(), 1);
    assert!(graph.contains_node(&name("a/1")));
    assert!(graph.contains_node(&name("b/1")));
}

#[test]
fn inserting_the_same_node_twice_does_not_duplicate_it() {
    let mut graph = Graph::new();
    graph.insert_edge(Edge::new(name("a/1"), name("b/1"), "MENTIONS", 1.0));
    graph.insert_edge(Edge::new(name("a/1"), name("c/1"), "MENTIONS", 1.0));
    assert_eq!(graph.node_count(), 3, "a/1 is shared, not duplicated");
    assert_eq!(graph.edge_count(), 2);
}

#[test]
fn two_edge_types_between_the_same_pair_both_survive() {
    let mut graph = Graph::new();
    graph.insert_edge(Edge::new(name("a/1"), name("b/1"), "MENTIONS", 1.0));
    graph.insert_edge(Edge::new(name("a/1"), name("b/1"), "CO_OCCURS", 1.0));
    assert_eq!(
        graph.node_count(),
        2,
        "still one node per endpoint, not one per edge"
    );
    assert_eq!(
        graph.edge_count(),
        2,
        "a GraphMap would have collapsed these"
    );
}

#[test]
fn an_unknown_node_is_not_contained() {
    let graph = Graph::new();
    assert!(!graph.contains_node(&name("a/1")));
}
