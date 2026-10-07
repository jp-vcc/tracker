//! Application state: the loaded data, the boot state and the one place where a
//! mutation is validated, persisted and swapped in (`apply`).

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::Serialize;

use crate::error::AppError;
use crate::model::{AppData, Snapshot};
use crate::store::{self, LoadOutcome};

#[derive(Debug, Clone, PartialEq)]
pub enum BootState {
    Ok,
    Corrupt { backup_path: String, reason: String },
    Newer { version: u32 },
    IoError { message: String },
}

/// Result of `bootstrap` / `retry_load`, serialised for the UI.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum BootResult {
    Ok {
        snapshot: Snapshot,
    },
    Corrupt {
        #[serde(rename = "backupPath")]
        backup_path: String,
        reason: String,
    },
    Newer {
        version: u32,
    },
    IoError {
        message: String,
    },
}

#[derive(Debug)]
pub struct AppState {
    pub data: AppData,
    pub boot: BootState,
    pub data_path: Option<PathBuf>,
}

impl AppState {
    /// Loads the store. Never fails: problems become a non-`Ok` boot state.
    pub fn boot(path: Result<PathBuf, String>, now_ms: i64) -> AppState {
        match path {
            Err(reason) => AppState {
                data: AppData::default(),
                boot: BootState::IoError {
                    message: format!("Could not find the app data folder ({reason})."),
                },
                data_path: None,
            },
            Ok(path) => {
                let (data, boot) = match store::load(&path, now_ms) {
                    LoadOutcome::Ok(d) => (d, BootState::Ok),
                    LoadOutcome::Empty => (AppData::default(), BootState::Ok),
                    LoadOutcome::Corrupt {
                        backup_path,
                        reason,
                    } => (
                        AppData::default(),
                        BootState::Corrupt {
                            backup_path: backup_path.to_string_lossy().into_owned(),
                            reason,
                        },
                    ),
                    LoadOutcome::Newer { version } => {
                        (AppData::default(), BootState::Newer { version })
                    }
                    LoadOutcome::IoError(message) => {
                        (AppData::default(), BootState::IoError { message })
                    }
                };
                AppState {
                    data,
                    boot,
                    data_path: Some(path),
                }
            }
        }
    }

    pub fn bootstrap(&self) -> BootResult {
        match &self.boot {
            BootState::Ok => BootResult::Ok {
                snapshot: self.data.clone(),
            },
            BootState::Corrupt {
                backup_path,
                reason,
            } => BootResult::Corrupt {
                backup_path: backup_path.clone(),
                reason: reason.clone(),
            },
            BootState::Newer { version } => BootResult::Newer { version: *version },
            BootState::IoError { message } => BootResult::IoError {
                message: message.clone(),
            },
        }
    }

    /// Reloads from a freshly resolved path. Valid only after an I/O error.
    pub fn retry_load(
        &mut self,
        path: Result<PathBuf, String>,
        now_ms: i64,
    ) -> Result<BootResult, AppError> {
        if !matches!(self.boot, BootState::IoError { .. }) {
            return Err(AppError::invalid_state("There is nothing to retry."));
        }
        *self = AppState::boot(path, now_ms);
        Ok(self.bootstrap())
    }

    /// Replaces unreadable or too-new data with an empty data set.
    /// Valid only in the `Corrupt` and `Newer` states.
    pub fn start_fresh(&mut self, now_ms: i64) -> Result<Snapshot, AppError> {
        let path = match (&self.boot, &self.data_path) {
            (BootState::Corrupt { .. } | BootState::Newer { .. }, Some(p)) => p.clone(),
            _ => {
                return Err(AppError::invalid_state(
                    "Starting with empty data is only possible after a data problem.",
                ))
            }
        };
        if matches!(self.boot, BootState::Newer { .. }) {
            // A newer file was not backed up at load time; do it before replacing it.
            let bytes = fs::read(&path).map_err(|_| {
                AppError::persistence("Could not read the data file to back it up.")
            })?;
            store::backup_newer(store::data_dir(&path), &bytes, now_ms).map_err(|_| {
                AppError::persistence("Could not back up the data file. It was not changed.")
            })?;
        }
        let fresh = AppData::default();
        save_to(&path, &fresh)?;
        self.data = fresh;
        self.boot = BootState::Ok;
        Ok(self.data.clone())
    }
}

fn save_to(path: &Path, data: &AppData) -> Result<(), AppError> {
    store::save(path, data).map_err(|_| {
        AppError::persistence("Could not save your changes. Nothing was changed.")
    })
}

/// The single validate -> persist -> swap sequence. Holds the lock for the whole
/// operation, refuses to run unless the boot state is `Ok`, and leaves memory
/// unchanged when either the rule or the write fails.
pub fn apply<F>(state: &Mutex<AppState>, op: F) -> Result<Snapshot, AppError>
where
    F: FnOnce(&AppData) -> Result<AppData, AppError>,
{
    let mut guard = state.lock().unwrap_or_else(|e| e.into_inner());
    if guard.boot != BootState::Ok {
        return Err(AppError::recovery_required());
    }
    let next = op(&guard.data)?;
    if next == guard.data {
        return Ok(guard.data.clone());
    }
    let path = guard
        .data_path
        .clone()
        .ok_or_else(AppError::recovery_required)?;
    save_to(&path, &next)?;
    guard.data = next;
    Ok(guard.data.clone())
}

