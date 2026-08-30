use super::MemoryStore;
use crate::SecretStore;

#[test]
fn a_stored_secret_reads_back_and_clears() {
    let store = MemoryStore::new();
    assert!(!store.has("openai"));
    assert_eq!(store.get("openai").expect("read"), None);

    store.set("openai", "sk-test").expect("write");
    assert!(store.has("openai"));
    assert_eq!(
        store.get("openai").expect("read").as_deref(),
        Some("sk-test")
    );

    store.set("openai", "sk-replaced").expect("overwrite");
    assert_eq!(
        store.get("openai").expect("read").as_deref(),
        Some("sk-replaced")
    );

    store.clear("openai").expect("clear");
    assert!(!store.has("openai"));
}

#[test]
fn clearing_something_that_was_never_there_is_not_an_error() {
    // The caller wanted it gone, and it is. Failing here would make a
    // "remove key" button fail on the one path where nothing was wrong.
    MemoryStore::new()
        .clear("never-set")
        .expect("clearing an absent key");
}

#[test]
fn providers_do_not_share_a_slot() {
    let store = MemoryStore::new();
    store.set("openai", "a").expect("set");
    store.set("anthropic", "b").expect("set");
    assert_eq!(store.get("openai").expect("read").as_deref(), Some("a"));
    assert_eq!(store.get("anthropic").expect("read").as_deref(), Some("b"));
}
