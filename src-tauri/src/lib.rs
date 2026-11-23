pub mod auth;
pub mod commands;
pub mod error;
pub mod events;
pub mod paths;
pub mod state;

use tauri::Manager;

use crate::paths::AppPaths;
use crate::state::AppState;

pub fn run() {
    init_tracing();

    let outcome = tauri::Builder::default()
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .map_err(|err| format!("failed to resolve app data directory: {err}"))?;

            let paths = AppPaths::new(data_dir);
            paths.ensure_layout()?;
            tracing::info!(root = %paths.root().display(), "app paths ready");

            app.manage(AppState::new(paths)?);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::health::app_info,
            commands::account::account_state,
            commands::account::create_offline_account,
            commands::account::remove_account,
            commands::account::set_active_account,
            commands::account::debug_seed_licensed_account,
        ])
        .run(tauri::generate_context!());

    if let Err(err) = outcome {
        tracing::error!(error = %err, "tauri runtime terminated");
        std::process::exit(1);
    }
}

fn init_tracing() {
    use tracing_subscriber::{fmt, EnvFilter};

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("orbit_client=debug,info"));

    // Повторная инициализация (например, в тестах) не должна ронять процесс.
    let _ = fmt().with_env_filter(filter).with_target(false).try_init();
}
