use super::*;

fn response(complete: bool, locked: Vec<String>) -> wire::SearchPointsResponse {
    wire::SearchPointsResponse {
        results: vec![wire::SearchResult {
            point: Some(wire::Point {
                name: "collections/docs/points/doc-1".to_owned(),
                vectors: Vec::new(),
                span: None,
                content_ref: Some(wire::ContentRef {
                    uri: String::new(),
                    range_start: 0,
                    range_end: 0,
                    sha256: Default::default(),
                    inline_text: "the cat sat".to_owned(),
                }),
            }),
            score: 0.9,
        }],
        next_page_token: String::new(),
        complete,
        answered_source_count: 1,
        total_source_count: 1,
        locked_vaults: locked,
        stats: None,
    }
}

#[test]
fn a_hit_carries_the_id_a_caller_can_pass_back() {
    // Not the full resource name: every other method takes an id, and a search
    // result that needed reformatting first would be the odd one out.
    let results = SearchResults::from_wire(response(true, Vec::new()));
    assert_eq!(results.hits()[0].name, "doc-1");
    assert_eq!(results.hits()[0].score, 0.9);
    assert_eq!(results.hits()[0].text.as_deref(), Some("the cat sat"));
}

#[test]
fn an_incomplete_search_says_so_rather_than_looking_like_a_full_one() {
    // The distinction rules 27 and 49 require: "no results" must be
    // distinguishable from "no results you can currently see".
    let results = SearchResults::from_wire(response(false, vec!["vault-a".to_owned()]));
    assert!(!results.is_complete());
    assert_eq!(results.locked_vaults(), &["vault-a".to_owned()]);
}

#[test]
fn a_complete_search_reports_full_coverage() {
    let results = SearchResults::from_wire(response(true, Vec::new()));
    assert!(results.is_complete());
    assert!(results.locked_vaults().is_empty());
    assert_eq!(results.coverage(), (1, 1));
}

#[test]
fn a_result_without_a_point_is_skipped_rather_than_panicking() {
    // `point` is optional on the wire, so a malformed or filtered entry must
    // not take the whole response down.
    let mut wire_response = response(true, Vec::new());
    wire_response.results.push(wire::SearchResult {
        point: None,
        score: 0.5,
    });

    let results = SearchResults::from_wire(wire_response);
    assert_eq!(results.len(), 1);
}

#[test]
fn results_iterate_directly() {
    let results = SearchResults::from_wire(response(true, Vec::new()));
    let names: Vec<&str> = (&results).into_iter().map(|h| h.name.as_str()).collect();
    assert_eq!(names, vec!["doc-1"]);
}
