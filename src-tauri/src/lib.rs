pub mod cli;
pub mod commands;
pub mod steam;
pub mod worker;

use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

use crate::commands::AppState;
use crate::worker::BackgroundWorker;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Check if invoked in headless CLI mode
    if cli::try_run_cli() {
        return;
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let _ = crate::steam::storage::init_db();
            let stored_settings = crate::steam::storage::load_settings();
            let stored_games = crate::steam::storage::load_cached_games();

            let handle = app.handle().clone();
            let worker = BackgroundWorker::new(handle);
            let state = AppState {
                settings: Arc::new(Mutex::new(stored_settings)),
                cached_games: Arc::new(Mutex::new(stored_games)),
                worker,
            };
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::save_settings,
            commands::detect_steam,
            commands::get_accounts,
            commands::load_library,
            commands::run_autocat,
            commands::save_to_steam,
            commands::get_categories,
            commands::verify_api_key,
            commands::fetch_hltb,
            commands::save_hltb_times,
            commands::get_autocat_rules,
            commands::save_autocat_rules,
        ])
        .run(tauri::generate_context!())
        .expect("error while running depressurizer application");
}
