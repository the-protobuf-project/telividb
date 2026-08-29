//! The composition root: where the engine, the commands and the window meet.
//!
//! Everything here is wiring. The engine is supervised by
//! `telividb-desktop-engine`, the commands live in `telividb-desktop-ipc`, and
//! neither knows about the other — this file is the only place that does, and
//! the only place that decides where the data directory is or which port to
//! use.

mod settings;

use settings::Settings;
use telividb_desktop_engine::Engine;
use telividb_desktop_ipc::AppState;

/// Start the engine, then open the window.
///
/// Blocks until the window closes, at which point `AppState` is dropped, the
/// shutdown sender goes with it, and the server stops through the same path a
/// deliberate stop uses.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let settings = Settings::resolve();

    // A dedicated runtime, started before Tauri.
    //
    // The engine has to be reachable before the first command can be invoked,
    // and a command that raced the engine's startup would fail for a reason the
    // window could do nothing about. Paying for it here means the window opens
    // knowing the answer.
    let runtime = tokio::runtime::Runtime::new().expect("a tokio runtime");
    let engine = match runtime.block_on(settings.start()) {
        Ok(engine) => engine,
        Err(error) => {
            // Before there is a window to show it in. Stderr is what a user
            // launching from a terminal sees, and what a crash reporter keeps.
            eprintln!("telividb could not start.\n\n{error}");
            std::process::exit(1);
        }
    };

    // The runtime outlives this function: the engine's background task is on
    // it, and dropping it here would stop the server the instant the window
    // opened.
    std::mem::forget(runtime);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::new(engine))
        .invoke_handler(telividb_desktop_ipc::commands!())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Re-exported so the binary can name it.
pub use telividb_desktop_engine::Error as EngineError;

/// The engine type, for callers wiring their own runtime.
pub type DesktopEngine = Engine;
