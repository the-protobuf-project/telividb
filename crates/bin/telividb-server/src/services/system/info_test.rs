//! What the system report must always be true about.

use super::*;

/// The service, with nothing configured.
fn svc() -> SystemSvc {
    SystemSvc::new()
}

#[tokio::test]
async fn names_the_selected_backend_not_the_compiled_one() {
    let got = svc()
        .get_system(Request::new(wire::GetSystemRequest {
            name: NAME.to_owned(),
        }))
        .await
        .expect("system is always describable")
        .into_inner();

    // The whole point of the service: a fallback to the host must be visible
    // rather than inferred from how slow things feel.
    assert!(
        !got.backend.is_empty(),
        "a backend is always selected, even if it is the host"
    );
    assert_eq!(got.name, NAME);
    assert_eq!(got.version, env!("CARGO_PKG_VERSION"));
}

#[tokio::test]
async fn budget_source_is_never_unspecified() {
    let got = svc()
        .get_system(Request::new(wire::GetSystemRequest::default()))
        .await
        .expect("an empty name means the singleton")
        .into_inner();

    // `UNSPECIFIED` is documented as never valid in a response, and a reader
    // cannot tell a measured ceiling from a guessed one without it.
    assert_ne!(got.budget_source, wire::BudgetSource::Unspecified as i32);
}

#[tokio::test]
async fn used_never_exceeds_the_limit_it_is_reported_against() {
    let got = svc()
        .get_system(Request::new(wire::GetSystemRequest::default()))
        .await
        .expect("describable")
        .into_inner();

    assert!(got.budget_limit_bytes >= 0, "a ceiling is never negative");
    assert!(got.budget_used_bytes >= 0, "residency is never negative");
    if got.budget_limit_bytes > 0 {
        assert!(
            got.budget_used_bytes <= got.budget_limit_bytes,
            "reported {} bytes held against a {} byte ceiling",
            got.budget_used_bytes,
            got.budget_limit_bytes
        );
    }
}

#[tokio::test]
async fn a_wrong_name_is_refused_rather_than_answered() {
    let refused = svc()
        .get_system(Request::new(wire::GetSystemRequest {
            name: "systems/other-machine".to_owned(),
        }))
        .await;

    // Answering for a name the caller did not ask about would let a client
    // believe it had reached a different machine.
    let status = refused.expect_err("a foreign name has no answer here");
    assert_eq!(status.code(), tonic::Code::NotFound);
}
