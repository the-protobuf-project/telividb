use super::*;

#[test]
fn a_fresh_set_has_visited_nothing() {
    let mut v = VisitedSet::with_capacity(8);
    v.clear();
    assert!(v.visit(0));
    assert!(v.visit(7));
}

#[test]
fn visiting_twice_reports_false_the_second_time() {
    let mut v = VisitedSet::with_capacity(8);
    v.clear();
    assert!(v.visit(3));
    assert!(!v.visit(3));
}

#[test]
fn clear_forgets_everything() {
    let mut v = VisitedSet::with_capacity(8);
    v.clear();
    v.visit(3);
    v.clear();
    assert!(v.visit(3), "a new search must see row 3 as unvisited");
}

#[test]
fn out_of_range_rows_are_not_a_panic() {
    let mut v = VisitedSet::with_capacity(4);
    v.clear();
    assert!(!v.visit(99));
}

#[test]
fn epoch_wraparound_does_not_leak_stale_visits() {
    // Stale stamps aliasing a recycled epoch would make a search skip nodes it
    // never actually visited, silently costing recall.
    let mut v = VisitedSet::with_capacity(4);
    v.epoch = u32::MAX - 1;
    v.clear();
    v.visit(2);
    v.clear();
    assert!(v.visit(2), "row 2 must look unvisited after the wrap");
}
