//! The join, on a graph small enough to reason about by hand.

use super::{Expansion, Seed};
use telividb_core::{Edge, ResourceName};
use telividb_graph::Graph;

fn name(s: &str) -> ResourceName {
    ResourceName::parse(format!("collections/c/points/{s}")).unwrap()
}

/// `doc` --MENTIONS--> `entity` --MENTIONS--> `far`, plus an unrelated `other`.
fn graph() -> Graph {
    let mut g = Graph::new();
    g.insert_edge(Edge::new(name("doc"), name("entity"), "MENTIONS", 1.0));
    g.insert_edge(Edge::new(name("entity"), name("far"), "MENTIONS", 1.0));
    g.insert_edge(Edge::new(name("doc"), name("other"), "CITES", 1.0));
    g
}

#[test]
fn a_seed_alone_comes_back_when_nothing_is_expanded() {
    let out = Expansion::default()
        .hops(0)
        .join(&[Seed::new(name("doc"), 0.9)], &graph(), None);

    assert_eq!(out.len(), 1);
    assert!(out[0].is_seed(), "hops=0 must return the search unchanged");
}

#[test]
fn expansion_reaches_neighbours_and_scores_them_below_the_seed() {
    let out = Expansion::default().join(&[Seed::new(name("doc"), 0.9)], &graph(), None);

    assert_eq!(out[0].name, name("doc"), "the direct hit still ranks first");
    assert!(out.len() > 1, "the graph contributed nothing");
    for hit in out.iter().filter(|h| !h.is_seed()) {
        assert!(hit.score < 0.9, "a neighbour outranked its own seed");
        assert_eq!(hit.via, Some(name("doc")), "provenance must name the seed");
        assert_eq!(hit.hops, 1);
    }
}

#[test]
fn an_edge_type_restricts_what_is_followed() {
    let out =
        Expansion::default()
            .along("MENTIONS")
            .join(&[Seed::new(name("doc"), 0.9)], &graph(), None);

    let names: Vec<&ResourceName> = out.iter().map(|h| &h.name).collect();
    assert!(names.contains(&&name("entity")));
    assert!(
        !names.contains(&&name("other")),
        "a CITES edge was followed under an edge_type of MENTIONS"
    );
}

#[test]
fn distance_compounds_the_decay() {
    let out = Expansion::default().hops(2).along("MENTIONS").join(
        &[Seed::new(name("doc"), 1.0)],
        &graph(),
        None,
    );

    let one = out.iter().find(|h| h.name == name("entity")).unwrap();
    let two = out.iter().find(|h| h.name == name("far")).unwrap();
    assert!(
        two.score < one.score,
        "two hops out scored {} against one hop at {}",
        two.score,
        one.score
    );
    assert_eq!(two.hops, 2);
}

#[test]
fn a_seed_is_never_displaced_by_the_graph_reaching_it() {
    // `entity` is both a direct hit and one hop from `doc`. The searched score
    // must survive — otherwise a document is demoted for being well connected.
    let out = Expansion::default().join(
        &[Seed::new(name("doc"), 0.9), Seed::new(name("entity"), 0.8)],
        &graph(),
        None,
    );

    let entity = out.iter().find(|h| h.name == name("entity")).unwrap();
    assert!(entity.is_seed(), "the graph overwrote a direct hit");
    assert_eq!(entity.score, 0.8);
}

#[test]
fn a_node_reached_twice_keeps_its_best_path_rather_than_the_sum() {
    // Summing would reward whichever node has the most inbound edges, which is
    // popularity rather than relevance — a hub would top every result.
    let mut g = Graph::new();
    g.insert_edge(Edge::new(name("a"), name("hub"), "L", 1.0));
    g.insert_edge(Edge::new(name("b"), name("hub"), "L", 1.0));

    let out = Expansion::default().decaying(0.5).join(
        &[Seed::new(name("a"), 0.4), Seed::new(name("b"), 0.8)],
        &g,
        None,
    );

    let hub = out.iter().find(|h| h.name == name("hub")).unwrap();
    assert_eq!(
        hub.score, 0.4,
        "0.8 * 0.5 — the better path, not 0.6 summed"
    );
    assert_eq!(
        hub.via,
        Some(name("b")),
        "provenance must name the better path"
    );
}

#[test]
fn the_budget_bounds_what_a_hub_can_contribute() {
    let mut g = Graph::new();
    for i in 0..50 {
        g.insert_edge(Edge::new(name("seed"), name(&format!("n{i}")), "L", 1.0));
    }

    let out = Expansion::default()
        .limited_to(5)
        .join(&[Seed::new(name("seed"), 1.0)], &g, None);

    let expanded = out.iter().filter(|h| !h.is_seed()).count();
    assert!(expanded <= 5, "budget of 5 admitted {expanded}");
}

#[test]
fn a_hidden_node_is_unreachable_rather_than_merely_unreturned() {
    // Invariant 34: the graph is a second path to a row, and it has to be
    // checked the same way. A node the predicate rejects must also not be a
    // stepping stone — otherwise its existence leaks through its neighbours.
    let hide_entity = |n: &ResourceName| *n != name("entity");

    let out = Expansion::default().hops(2).along("MENTIONS").join(
        &[Seed::new(name("doc"), 1.0)],
        &graph(),
        Some(&hide_entity),
    );

    let names: Vec<&ResourceName> = out.iter().map(|h| &h.name).collect();
    assert!(!names.contains(&&name("entity")), "hidden node returned");
    assert!(
        !names.contains(&&name("far")),
        "a hidden node was still traversed *through* — this is the leak, not the miss"
    );
}

#[test]
fn results_are_ordered_reproducibly() {
    let seeds = [Seed::new(name("doc"), 0.9), Seed::new(name("entity"), 0.9)];
    let first = Expansion::default().join(&seeds, &graph(), None);
    let again = Expansion::default().join(&seeds, &graph(), None);
    assert_eq!(first, again, "identical queries returned different orders");
}
