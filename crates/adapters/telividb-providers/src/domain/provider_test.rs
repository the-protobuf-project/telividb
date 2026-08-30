use super::{Locality, PROVIDERS, provider};

#[test]
fn exactly_one_provider_is_local() {
    // The whole vault rule rests on this: a key-wrapped space is answerable
    // only by something on this machine, so there has to *be* one, and a second
    // would need its own review before it could carry that trust.
    let local: Vec<&str> = PROVIDERS
        .iter()
        .filter(|p| p.locality == Locality::Local)
        .map(|p| p.id)
        .collect();
    assert_eq!(local, vec!["ollama"]);
}

#[test]
fn every_remote_provider_needs_a_key_and_every_local_one_does_not() {
    for p in PROVIDERS {
        assert_eq!(
            p.needs_key(),
            !p.is_local(),
            "{}: a provider's locality is what decides whether it needs a key",
            p.id
        );
    }
}

#[test]
fn ids_are_unique_and_every_provider_offers_a_model() {
    let mut ids: Vec<&str> = PROVIDERS.iter().map(|p| p.id).collect();
    let count = ids.len();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), count, "two providers share an id");

    for p in PROVIDERS {
        assert!(!p.models.is_empty(), "{}: offers nothing to select", p.id);
        assert!(!p.credential_hint.is_empty(), "{}", p.id);
    }
}

#[test]
fn an_unknown_id_resolves_to_nothing() {
    assert!(provider("ollama").is_some());
    assert!(provider("not-a-provider").is_none());
}
