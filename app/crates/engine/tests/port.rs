//! Starting an engine on a port something else already holds.
//!
//! This exists because the failure it covers was found in the running product,
//! not in a test: a five-day-old build of this project was still listening on
//! the default port, the window connected to *it*, and every call went to a
//! server missing half the services. Nothing reported a problem — the symptom
//! was `Unimplemented` on a service this build definitely has.

use std::net::TcpListener;
use telividb_desktop_engine::{Engine, Error};

#[tokio::test]
async fn a_port_held_by_something_else_is_refused_rather_than_adopted() {
    // Stand in for the stale process: hold the port, and keep holding it.
    let squatter = TcpListener::bind("127.0.0.1:0").expect("a port to hold");
    let addr = squatter.local_addr().expect("its address");

    let dir = tempfile::tempdir().expect("temp data dir");
    let outcome = Engine::start(dir.path().to_path_buf(), addr, None).await;

    let Err(error) = outcome else {
        panic!(
            "the engine reported success while {addr} was held by another \
             process, which means it connected to that process instead"
        );
    };
    assert!(
        matches!(error, Error::PortBusy { .. }),
        "expected the port to be named as busy, got: {error}"
    );

    // The message has to be actionable: someone meeting this needs to find the
    // other process, and the port is the only handle they have on it.
    let said = error.to_string();
    assert!(said.contains(&addr.port().to_string()), "{said}");
    assert!(said.contains("lsof"), "{said}");

    drop(squatter);
}
