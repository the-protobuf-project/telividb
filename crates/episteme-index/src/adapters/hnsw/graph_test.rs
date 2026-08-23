use super::*;

#[test]
fn an_empty_graph_has_no_entry() {
    let g = Graph::new();
    assert!(g.is_empty());
    assert_eq!(g.entry(), None);
}

#[test]
fn the_first_node_becomes_the_entry() {
    let mut g = Graph::new();
    let n = g.push_node(0);
    assert_eq!(g.entry(), Some(Ordinal::from_row(n)));
    assert_eq!(g.max_level(), 0);
}

#[test]
fn a_higher_node_takes_over_as_entry() {
    // Descent must start from the top, or the upper layers are unreachable.
    let mut g = Graph::new();
    g.push_node(0);
    g.push_node(2);
    g.push_node(1);
    assert_eq!(g.entry(), Some(Ordinal::from_row(1)));
    assert_eq!(g.max_level(), 2);
}

#[test]
fn nodes_occupy_every_layer_up_to_their_level() {
    let mut g = Graph::new();
    let n = g.push_node(2);
    g.set_neighbours(n, 0, vec![1, 2]);
    g.set_neighbours(n, 2, vec![3]);
    assert_eq!(g.neighbours(n, 0), &[1, 2]);
    assert_eq!(g.neighbours(n, 2), &[3]);
    assert_eq!(g.neighbours(n, 3), &[] as &[u32], "above its level");
}

#[test]
fn neighbours_of_an_unknown_node_are_empty_not_a_panic() {
    let g = Graph::new();
    assert_eq!(g.neighbours(99, 0), &[] as &[u32]);
    assert_eq!(g.level_of(99), 0);
}

#[test]
fn connect_reports_when_the_budget_is_reached() {
    let mut g = Graph::new();
    let n = g.push_node(0);
    assert!(!g.try_connect(n, 0, 1, 2));
    assert!(
        g.try_connect(n, 0, 2, 2),
        "second edge fills a budget of two"
    );
}

#[test]
fn connect_is_idempotent() {
    let mut g = Graph::new();
    let n = g.push_node(0);
    g.try_connect(n, 0, 1, 4);
    g.try_connect(n, 0, 1, 4);
    assert_eq!(g.neighbours(n, 0), &[1], "no duplicate edge");
}

#[test]
fn edges_are_counted_across_layers() {
    let mut g = Graph::new();
    let a = g.push_node(1);
    g.set_neighbours(a, 0, vec![1, 2, 3]);
    g.set_neighbours(a, 1, vec![4]);
    assert_eq!(g.edge_count(), 4);
}
