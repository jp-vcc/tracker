mod commands;

use std::sync::Mutex;

use tauri::Manager;
use tauri_plugin_log::{Target, TargetKind};
use tracker_core::clock::{Clock, SystemClock};
use tracker_core::state::{AppState, BootState};

pub fn run() {
    tauri::Builder::default()
        // Registered first so a second process exits before it loads the store.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Warn)
                .targets([Target::new(TargetKind::LogDir { file_name: None })])
                .build(),
        )
        .setup(|app| {
            // Never abort here: a failure becomes a boot state the UI can show.
            let path = commands::resolve_data_path(app.handle());
            if path.is_err() {
                log::error!("startup: app data path could not be resolved");
            }
            let state = AppState::boot(path, SystemClock.now_ms());
            match &state.boot {
                BootState::Ok => {}
                BootState::Corrupt { .. } => log::warn!("startup: data file is corrupt"),
                BootState::Newer { version } => {
                    log::warn!("startup: data file has newer schemaVersion {version}")
                }
                BootState::IoError { .. } => log::error!("startup: data file is unreadable"),
            }
            app.manage(Mutex::new(state));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::bootstrap,
            commands::retry_load,
            commands::start_fresh,
            commands::create_project,
            commands::rename_project,
            commands::delete_project,
            commands::create_task,
            commands::update_task,
            commands::delete_task,
            commands::start_timer,
            commands::stop_timer,
            commands::discard_timer,
            commands::add_entry,
            commands::update_entry,
            commands::delete_entry,
        ])
        .run(tauri::generate_context!())
        .expect("error while running the time tracker");
}

