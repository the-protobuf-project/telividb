use super::*;

#[test]
fn every_crate_in_the_workspace_has_a_declared_position() {
    // A new crate with no entry in ALLOWED is reported rather than skipped:
    // silently permitting whatever it depends on is how the direction gets
    // lost, one crate at a time.
    let root = std::env::current_dir().expect("cwd");
    let crates = find_crates(&root);
    if crates.is_empty() {
        return; // Run from somewhere without the workspace; nothing to assert.
    }
    let declared: Vec<&str> = ALLOWED.iter().map(|(name, _)| *name).collect();
    for dir in &crates {
        let name = dir
            .file_name()
            .expect("crate dir has a name")
            .to_string_lossy();
        assert!(
            declared.contains(&name.as_ref()),
            "{name} has no entry in check-layers::ALLOWED"
        );
    }
}

#[test]
fn core_depends_on_nothing_in_the_workspace() {
    // The property the whole layering rests on. If core grows a workspace
    // dependency, "dependencies point inward" has no fixed point left.
    let (_, permitted) = ALLOWED
        .iter()
        .find(|(name, _)| *name == "telividb-core")
        .expect("core is declared");
    assert!(
        permitted.is_empty(),
        "core may depend on nothing in-workspace"
    );
}

#[test]
fn storage_cannot_see_the_index() {
    // Invariant 6: the index talks to a `VectorStore`, so storage layout and
    // search algorithm evolve independently. An edge either way collapses that.
    let (_, storage) = ALLOWED
        .iter()
        .find(|(name, _)| *name == "telividb-storage")
        .expect("storage is declared");
    assert!(!storage.contains(&"telividb-index"));

    let (_, index) = ALLOWED
        .iter()
        .find(|(name, _)| *name == "telividb-index")
        .expect("index is declared");
    assert!(!index.contains(&"telividb-storage"));
}

#[test]
fn an_outward_dependency_is_reported() {
    let dir = std::env::temp_dir().join("xtask-check-layers-outward");
    std::fs::create_dir_all(&dir).expect("temp dir");
    let manifest = dir.join("Cargo.toml");
    std::fs::write(
        &manifest,
        "[package]\nname = \"telividb-core\"\n\n[dependencies]\ntelividb-index.workspace = true\n",
    )
    .expect("write manifest");

    let mut problems = Vec::new();
    check_manifest(&manifest, "telividb-core", &[], &mut problems);
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(problems[0].contains("telividb-index"));
}

#[test]
fn a_dev_dependency_is_allowed_to_point_outward() {
    // An integration test exercising the seam is not a layering violation.
    let dir = std::env::temp_dir().join("xtask-check-layers-dev");
    std::fs::create_dir_all(&dir).expect("temp dir");
    let manifest = dir.join("Cargo.toml");
    std::fs::write(
        &manifest,
        "[package]\nname = \"telividb-storage\"\n\n[dev-dependencies]\ntelividb-index.workspace = true\n",
    )
    .expect("write manifest");

    let mut problems = Vec::new();
    check_manifest(
        &manifest,
        "telividb-storage",
        &["telividb-core"],
        &mut problems,
    );
    assert!(problems.is_empty(), "{problems:?}");
}
