//! The composition root: where the engine, the commands and the window meet.
//!
//! Everything here is wiring. The engine is supervised by
//! `telividb-desktop-engine`, the commands live in `telividb-desktop-ipc`, and
//! neither knows about the other — this file is the only place that does, and
//! the only place that decides where the data directory is or which port to
//! use.

mod settings;

use settings::Settings;
use tauri::Manager;
use telividb_desktop_engine::Engine;
use telividb_desktop_ipc::AppState;

/// Start the engine, then open the window.
///
/// Shuts the engine down on `RunEvent::Exit` rather than by dropping
/// `AppState`, because on macOS `AppState` is never dropped: Cmd+Q reaches
/// `-[NSApplication terminate:]`, which calls `exit()` from inside the menu
/// action and never returns through `Builder::run`. `exit()` then runs ggml's
/// static destructors — its device list is a function-local `static
/// std::vector`, so it is registered with `__cxa_atexit` — and
/// `ggml_metal_rsets_free` asserts that every residency set was released
/// first. With the engine still holding its Metal buffers that assert fails,
/// and the app aborts with SIGABRT on every quit.
///
/// `applicationWillTerminate:` is the seam. AppKit sends it before `terminate:`
/// calls `exit()`; tao answers it with `Event::LoopDestroyed`, which arrives
/// here as `RunEvent::Exit`. So this handler is the last code that runs while
/// the process is still whole, and the only place a graceful teardown fits.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let settings = Settings::resolve();
    let has_model = settings.model.is_some();
    let data_dir = settings.data_dir.display().to_string();

    // A dedicated runtime, started before Tauri.
    //
    // The engine has to be reachable before the first command can be invoked,
    // and a command that raced the engine's startup would fail for a reason the
    // window could do nothing about. Paying for it here means the window opens
    // knowing the answer.
    let runtime = tokio::runtime::Runtime::new().expect("a tokio runtime");
    // Kept before the runtime is forgotten: quitting has to await the server
    // task, and a handle is the only way back onto a runtime nobody owns.
    let shutdown_on = runtime.handle().clone();
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

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_http::init())
        .manage(AppState::new(engine, has_model, data_dir))
        .invoke_handler(telividb_desktop_ipc::commands!())
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(move |app, event| {
        // `Exit` rather than `ExitRequested`: the latter is the window asking
        // whether to go, and can be cancelled. This one is the loop being
        // destroyed, which happens on every route out — the last window
        // closing, and `terminate:` alike.
        if !matches!(event, tauri::RunEvent::Exit) {
            return;
        }

        let Some(engine) = app.state::<AppState>().take_engine() else {
            return;
        };

        // Blocking here is the point. The handler is running inside
        // `applicationWillTerminate:`, so returning early would hand control
        // straight back to `exit()` with the buffers still live — which is the
        // crash this exists to prevent. `Handle::block_on` is safe from here
        // because the main thread is not one of the runtime's workers.
        shutdown_on.block_on(engine.shutdown());
    });
}

/// Re-exported so the binary can name it.
pub use telividb_desktop_engine::Error as EngineError;

/// The engine type, for callers wiring their own runtime.
pub type DesktopEngine = Engine;
