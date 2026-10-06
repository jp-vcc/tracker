use std::collections::HashSet;

use serde::{Deserialize, Serialize};

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    Timer,
    Manual,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub id: String,
    pub task_id: String,
    /// Local calendar date `YYYY-MM-DD`. Authoritative for "when".
    pub date: String,
    /// UTC epoch ms of the timer start. Audit only; `null` for manual entries.
    pub started_at: Option<i64>,
    pub duration_sec: i64,
    pub note: String,
    pub source: Source,
    pub created_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveTimer {
    pub task_id: String,
    pub started_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppData {
    pub schema_version: u32,
    pub projects: Vec<Project>,
    pub tasks: Vec<Task>,
    pub entries: Vec<Entry>,
    pub active_timer: Option<ActiveTimer>,
}

/// What the UI receives after every command.
pub type Snapshot = AppData;

impl Default for AppData {
    fn default() -> Self {
        AppData {
            schema_version: SCHEMA_VERSION,
            projects: Vec::new(),
            tasks: Vec::new(),
            entries: Vec::new(),
            active_timer: None,
        }
    }
}

/// True for a strict `YYYY-MM-DD` calendar date.
pub fn is_valid_date(s: &str) -> bool {
    s.len() == 10 && chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").is_ok()
}

/// Checks referential integrity and basic value rules of loaded data.
///
/// It does not cap durations (a timer entry over 24 h is legal) and does not
/// reject duplicate names, so a hand-edited file still loads.
pub fn validate_integrity(data: &AppData) -> Result<(), String> {
    let mut project_ids = HashSet::new();
    for p in &data.projects {
        if !project_ids.insert(p.id.as_str()) {
            return Err("duplicate project id".to_string());
        }
    }

    let mut task_ids = HashSet::new();
    for t in &data.tasks {
        if !task_ids.insert(t.id.as_str()) {
            return Err("duplicate task id".to_string());
        }
        if !project_ids.contains(t.project_id.as_str()) {
            return Err("a task references a missing project".to_string());
        }
    }

    let mut entry_ids = HashSet::new();
    for e in &data.entries {
        if !entry_ids.insert(e.id.as_str()) {
            return Err("duplicate entry id".to_string());
        }
        if !task_ids.contains(e.task_id.as_str()) {
            return Err("an entry references a missing task".to_string());
        }
        if e.duration_sec <= 0 {
            return Err("an entry has a non-positive duration".to_string());
        }
        if !is_valid_date(&e.date) {
            return Err("an entry has an invalid date".to_string());
        }
    }

    if let Some(t) = &data.active_timer {
        if !task_ids.contains(t.task_id.as_str()) {
            return Err("the active timer references a missing task".to_string());
        }
    }
    Ok(())
}

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    Timer,
    Manual,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub id: String,
    pub task_id: String,
    /// Local calendar date `YYYY-MM-DD`. Authoritative for "when".
    pub date: String,
    /// UTC epoch ms of the timer start. Audit only; `null` for manual entries.
    pub started_at: Option<i64>,
    pub duration_sec: i64,
    pub note: String,
    pub source: Source,
    pub created_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveTimer {
    pub task_id: String,
    pub started_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppData {
    pub schema_version: u32,
    pub projects: Vec<Project>,
    pub tasks: Vec<Task>,
    pub entries: Vec<Entry>,
    pub active_timer: Option<ActiveTimer>,
}

/// What the UI receives after every command.
pub type Snapshot = AppData;

impl Default for AppData {
    fn default() -> Self {
        AppData {
            schema_version: SCHEMA_VERSION,
            projects: Vec::new(),
            tasks: Vec::new(),
            entries: Vec::new(),
            active_timer: None,
        }
    }
}

/// True for a strict `YYYY-MM-DD` calendar date.
pub fn is_valid_date(s: &str) -> bool {
    s.len() == 10 && chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").is_ok()
}

/// Checks referential integrity and basic value rules of loaded data.
///
/// It does not cap durations (a timer entry over 24 h is legal) and does not
/// reject duplicate names, so a hand-edited file still loads.
pub fn validate_integrity(data: &AppData) -> Result<(), String> {
    let mut project_ids = HashSet::new();
    for p in &data.projects {
        if !project_ids.insert(p.id.as_str()) {
            return Err("duplicate project id".to_string());
        }
    }

    let mut task_ids = HashSet::new();
    for t in &data.tasks {
        if !task_ids.insert(t.id.as_str()) {
            return Err("duplicate task id".to_string());
        }
        if !project_ids.contains(t.project_id.as_str()) {
            return Err("a task references a missing project".to_string());
        }
    }

    let mut entry_ids = HashSet::new();
    for e in &data.entries {
        if !entry_ids.insert(e.id.as_str()) {
            return Err("duplicate entry id".to_string());
        }
        if !task_ids.contains(e.task_id.as_str()) {
            return Err("an entry references a missing task".to_string());
        }
        if e.duration_sec <= 0 {
            return Err("an entry has a non-positive duration".to_string());
        }
        if !is_valid_date(&e.date) {
            return Err("an entry has an invalid date".to_string());
        }
    }

    if let Some(t) = &data.active_timer {
        if !task_ids.contains(t.task_id.as_str()) {
            return Err("the active timer references a missing task".to_string());
        }
    }
    Ok(())
}

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    Timer,
    Manual,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub id: String,
    pub task_id: String,
    /// Local calendar date `YYYY-MM-DD`. Authoritative for "when".
    pub date: String,
    /// UTC epoch ms of the timer start. Audit only; `null` for manual entries.
    pub started_at: Option<i64>,
    pub duration_sec: i64,
    pub note: String,
    pub source: Source,
    pub created_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveTimer {
    pub task_id: String,
    pub started_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppData {
    pub schema_version: u32,
    pub projects: Vec<Project>,
    pub tasks: Vec<Task>,
    pub entries: Vec<Entry>,
    pub active_timer: Option<ActiveTimer>,
}

/// What the UI receives after every command.
pub type Snapshot = AppData;

impl Default for AppData {
    fn default() -> Self {
        AppData {
            schema_version: SCHEMA_VERSION,
            projects: Vec::new(),
            tasks: Vec::new(),
            entries: Vec::new(),
            active_timer: None,
        }
    }
}

/// True for a strict `YYYY-MM-DD` calendar date.
pub fn is_valid_date(s: &str) -> bool {
    s.len() == 10 && chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").is_ok()
}

/// Checks referential integrity and basic value rules of loaded data.
///
/// It does not cap durations (a timer entry over 24 h is legal) and does not
/// reject duplicate names, so a hand-edited file still loads.
pub fn validate_integrity(data: &AppData) -> Result<(), String> {
    let mut project_ids = HashSet::new();
    for p in &data.projects {
        if !project_ids.insert(p.id.as_str()) {
            return Err("duplicate project id".to_string());
        }
    }

    let mut task_ids = HashSet::new();
    for t in &data.tasks {
        if !task_ids.insert(t.id.as_str()) {
            return Err("duplicate task id".to_string());
        }
        if !project_ids.contains(t.project_id.as_str()) {
            return Err("a task references a missing project".to_string());
        }
    }

    let mut entry_ids = HashSet::new();
    for e in &data.entries {
        if !entry_ids.insert(e.id.as_str()) {
            return Err("duplicate entry id".to_string());
        }
        if !task_ids.contains(e.task_id.as_str()) {
            return Err("an entry references a missing task".to_string());
        }
        if e.duration_sec <= 0 {
            return Err("an entry has a non-positive duration".to_string());
        }
        if !is_valid_date(&e.date) {
            return Err("an entry has an invalid date".to_string());
        }
    }

    if let Some(t) = &data.active_timer {
        if !task_ids.contains(t.task_id.as_str()) {
            return Err("the active timer references a missing task".to_string());
        }
    }
    Ok(())
}

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    Timer,
    Manual,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub id: String,
    pub task_id: String,
    /// Local calendar date `YYYY-MM-DD`. Authoritative for "when".
    pub date: String,
    /// UTC epoch ms of the timer start. Audit only; `null` for manual entries.
    pub started_at: Option<i64>,
    pub duration_sec: i64,
    pub note: String,
    pub source: Source,
    pub created_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveTimer {
    pub task_id: String,
    pub started_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppData {
    pub schema_version: u32,
    pub projects: Vec<Project>,
    pub tasks: Vec<Task>,
    pub entries: Vec<Entry>,
    pub active_timer: Option<ActiveTimer>,
}

/// What the UI receives after every command.
pub type Snapshot = AppData;

impl Default for AppData {
    fn default() -> Self {
        AppData {
            schema_version: SCHEMA_VERSION,
            projects: Vec::new(),
            tasks: Vec::new(),
            entries: Vec::new(),
            active_timer: None,
        }
    }
}

/// True for a strict `YYYY-MM-DD` calendar date.
pub fn is_valid_date(s: &str) -> bool {
    s.len() == 10 && chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").is_ok()
}

/// Checks referential integrity and basic value rules of loaded data.
///
/// It does not cap durations (a timer entry over 24 h is legal) and does not
/// reject duplicate names, so a hand-edited file still loads.
pub fn validate_integrity(data: &AppData) -> Result<(), String> {
    let mut project_ids = HashSet::new();
    for p in &data.projects {
        if !project_ids.insert(p.id.as_str()) {
            return Err("duplicate project id".to_string());
        }
    }

    let mut task_ids = HashSet::new();
    for t in &data.tasks {
        if !task_ids.insert(t.id.as_str()) {
            return Err("duplicate task id".to_string());
        }
        if !project_ids.contains(t.project_id.as_str()) {
            return Err("a task references a missing project".to_string());
        }
    }

    let mut entry_ids = HashSet::new();
    for e in &data.entries {
        if !entry_ids.insert(e.id.as_str()) {
            return Err("duplicate entry id".to_string());
        }
        if !task_ids.contains(e.task_id.as_str()) {
            return Err("an entry references a missing task".to_string());
        }
        if e.duration_sec <= 0 {
            return Err("an entry has a non-positive duration".to_string());
        }
        if !is_valid_date(&e.date) {
            return Err("an entry has an invalid date".to_string());
        }
    }

    if let Some(t) = &data.active_timer {
        if !task_ids.contains(t.task_id.as_str()) {
            return Err("the active timer references a missing task".to_string());
        }
    }
    Ok(())
}

