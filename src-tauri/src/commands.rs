//! Thin Tauri handlers. Every mutation goes through `tracker_core::state::apply`;
//! no rule or persistence logic lives here.

use std::path::PathBuf;
use std::sync::Mutex;

use tauri::{AppHandle, Manager, State};
use tracker_core::clock::{Clock, SystemClock};
use tracker_core::error::AppError;
use tracker_core::model::Snapshot;
use tracker_core::service::{self, EntryPatch, NewEntry, TaskPatch};
use tracker_core::state::{apply, AppState, BootResult};
use tracker_core::store::DATA_FILE;

type Shared<'a> = State<'a, Mutex<AppState>>;
type CmdResult = Result<Snapshot, AppError>;

/// `<app_data_dir>/data.json`, i.e. `%APPDATA%\dev.tracker.timetracker\data.json` on Windows.
pub fn resolve_data_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|dir| dir.join(DATA_FILE))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn bootstrap(state: Shared) -> BootResult {
    state.lock().unwrap_or_else(|e| e.into_inner()).bootstrap()
}

#[tauri::command]
pub fn retry_load(app: AppHandle, state: Shared) -> Result<BootResult, AppError> {
    let path = resolve_data_path(&app);
    state
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .retry_load(path, SystemClock.now_ms())
}

#[tauri::command]
pub fn start_fresh(state: Shared) -> CmdResult {
    state
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .start_fresh(SystemClock.now_ms())
}

#[tauri::command]
pub fn create_project(state: Shared, name: String) -> CmdResult {
    apply(&state, |d| service::create_project(d, &name, &SystemClock))
}

#[tauri::command]
pub fn rename_project(state: Shared, id: String, name: String) -> CmdResult {
    apply(&state, |d| service::rename_project(d, &id, &name))
}

#[tauri::command]
pub fn delete_project(state: Shared, id: String) -> CmdResult {
    apply(&state, |d| service::delete_project(d, &id))
}

#[tauri::command]
pub fn create_task(state: Shared, project_id: String, name: String) -> CmdResult {
    apply(&state, |d| {
        service::create_task(d, &project_id, &name, &SystemClock)
    })
}

#[tauri::command]
pub fn update_task(state: Shared, id: String, patch: TaskPatch) -> CmdResult {
    apply(&state, |d| service::update_task(d, &id, &patch))
}

#[tauri::command]
pub fn delete_task(state: Shared, id: String) -> CmdResult {
    apply(&state, |d| service::delete_task(d, &id))
}

#[tauri::command]
pub fn start_timer(state: Shared, task_id: String) -> CmdResult {
    apply(&state, |d| service::start_timer(d, &task_id, &SystemClock))
}

#[tauri::command]
pub fn stop_timer(state: Shared) -> CmdResult {
    apply(&state, |d| service::stop_timer(d, &SystemClock))
}

#[tauri::command]
pub fn discard_timer(state: Shared) -> CmdResult {
    apply(&state, service::discard_timer)
}

#[tauri::command]
pub fn add_entry(state: Shared, entry: NewEntry) -> CmdResult {
    apply(&state, |d| service::add_entry(d, &entry, &SystemClock))
}

#[tauri::command]
pub fn update_entry(state: Shared, id: String, patch: EntryPatch) -> CmdResult {
    apply(&state, |d| {
        service::update_entry(d, &id, &patch, &SystemClock)
    })
}

#[tauri::command]
pub fn delete_entry(state: Shared, id: String) -> CmdResult {
    apply(&state, |d| service::delete_entry(d, &id))
}

