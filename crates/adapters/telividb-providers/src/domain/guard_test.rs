use super::may_answer;
use crate::domain::provider;
use telividb_core::Protection;

const LOCAL: &str = "ollama";
const REMOTE: &str = "openai";

#[test]
fn a_private_space_may_use_any_provider() {
    for id in [LOCAL, REMOTE] {
        let p = provider(id).expect("a known provider");
        assert!(
            may_answer("spaces/notes", Protection::Private, p).is_ok(),
            "{id}"
        );
    }
}

#[test]
fn a_protected_space_may_only_use_a_local_provider() {
    let local = provider(LOCAL).expect("ollama");
    let remote = provider(REMOTE).expect("openai");

    for protection in [Protection::Vault, Protection::Sealed] {
        assert!(
            may_answer("spaces/board", protection, local).is_ok(),
            "a model on this machine takes nothing off it"
        );
        let err = may_answer("spaces/board", protection, remote)
            .expect_err("a remote provider must be refused, not warned about");
        let said = err.to_string();
        // The message has to name all three things a person needs to act:
        // which space, why, and what to do instead.
        assert!(said.contains("spaces/board"), "{said}");
        assert!(said.contains("OpenAI"), "{said}");
        assert!(said.contains("local provider"), "{said}");
    }
}

#[test]
fn a_sealed_space_says_sealed_rather_than_key_wrapped() {
    // The two are different promises and a reader should not have to guess
    // which one they are being told about.
    let remote = provider(REMOTE).expect("openai");
    let sealed = may_answer("s", Protection::Sealed, remote)
        .unwrap_err()
        .to_string();
    let vault = may_answer("s", Protection::Vault, remote)
        .unwrap_err()
        .to_string();
    assert!(sealed.contains("sealed"), "{sealed}");
    assert!(vault.contains("key-wrapped"), "{vault}");
}
