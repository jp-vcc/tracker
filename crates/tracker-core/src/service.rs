//! Domain rules. Every operation is a pure function `(&AppData, args, clock) ->
//! Result<AppData, AppError>`: it never mutates its input and does no I/O, so a
//! failed validation leaves the caller's data untouched.

use std::collections::HashSet;

use serde::Deserialize;
use uuid::Uuid;

use crate::clock::Clock;
use crate::error::AppError;
use crate::model::{is_valid_date, ActiveTimer, AppData, Entry, Project, Source, Task};

pub const MAX_DURATION_SEC: i64 = 86_400;
pub const MAX_NOTE_CHARS: usize = 500;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewEntry {
    pub task_id: String,
    pub date: String,
    pub duration_sec: i64,
    #[serde(default)]
    pub note: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskPatch {
    pub name: Option<String>,
    pub project_id: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryPatch {
    pub task_id: Option<String>,
    pub date: Option<String>,
    pub duration_sec: Option<i64>,
    pub note: Option<String>,
}

fn new_id() -> String {
    Uuid::new_v4().to_string()
}

fn clean_name(raw: &str) -> Result<String, AppError> {
    let name = raw.trim();
    if name.is_empty() {
        Err(AppError::validation("name", "Name is required"))
    } else {
        Ok(name.to_string())
    }
}

fn same_name(a: &str, b: &str) -> bool {
    a.to_lowercase() == b.to_lowercase()
}

fn check_project_name_free(
    data: &AppData,
    name: &str,
    except_id: Option<&str>,
) -> Result<(), AppError> {
    let taken = data
        .projects
        .iter()
        .any(|p| Some(p.id.as_str()) != except_id && same_name(&p.name, name));
    if taken {
        Err(AppError::duplicate(
            "name",
            "A project with this name already exists",
        ))
    } else {
        Ok(())
    }
}

fn check_task_name_free(
    data: &AppData,
    project_id: &str,
    name: &str,
    except_id: Option<&str>,
) -> Result<(), AppError> {
    let taken = data.tasks.iter().any(|t| {
        t.project_id == project_id && Some(t.id.as_str()) != except_id && same_name(&t.name, name)
    });
    if taken {
        Err(AppError::duplicate("name", "Name already exists in this project"))
    } else {
        Ok(())
    }
}

fn check_duration(sec: i64) -> Result<(), AppError> {
    if sec <= 0 {
        Err(AppError::validation(
            "duration",
            "Duration must be greater than zero",
        ))
    } else if sec > MAX_DURATION_SEC {
        Err(AppError::validation(
            "duration",
            "Duration cannot be more than 24 hours",
        ))
    } else {
        Ok(())
    }
}

fn check_date(date: &str, clock: &dyn Clock) -> Result<(), AppError> {
    if !is_valid_date(date) {
        return Err(AppError::validation("date", "Enter a valid date"));
    }
    let today = clock.today();
    if date > today.as_str() {
        return Err(AppError::validation(
            "date",
            "Date cannot be in the future",
        ));
    }
    Ok(())
}

fn check_note(note: &str) -> Result<String, AppError> {
    let note = note.trim();
    if note.chars().count() > MAX_NOTE_CHARS {
        return Err(AppError::validation(
            "note",
            "Note cannot be longer than 500 characters",
        ));
    }
    Ok(note.to_string())
}

/// Whole seconds between `started_at` and `now`, never negative.
pub fn elapsed_sec(started_at: i64, now: i64) -> i64 {
    (now - started_at).max(0) / 1000
}

// --- projects --------------------------------------------------------------

pub fn create_project(data: &AppData, name: &str, clock: &dyn Clock) -> Result<AppData, AppError> {
    let name = clean_name(name)?;
    check_project_name_free(data, &name, None)?;
    let mut next = data.clone();
    next.projects.push(Project {
        id: new_id(),
        name,
        created_at: clock.now_ms(),
    });
    Ok(next)
}

pub fn rename_project(data: &AppData, id: &str, name: &str) -> Result<AppData, AppError> {
    if !data.projects.iter().any(|p| p.id == id) {
        return Err(AppError::not_found("project"));
    }
    let name = clean_name(name)?;
    check_project_name_free(data, &name, Some(id))?;
    let mut next = data.clone();
    if let Some(p) = next.projects.iter_mut().find(|p| p.id == id) {
        p.name = name;
    }
    Ok(next)
}

/// Removes the given tasks, their entries, and the active timer if it points at one of them.
fn remove_tasks(data: &mut AppData, task_ids: &HashSet<String>) {
    data.tasks.retain(|t| !task_ids.contains(&t.id));
    data.entries.retain(|e| !task_ids.contains(&e.task_id));
    let timer_removed = data
        .active_timer
        .as_ref()
        .map_or(false, |t| task_ids.contains(&t.task_id));
    if timer_removed {
        data.active_timer = None;
    }
}

pub fn delete_project(data: &AppData, id: &str) -> Result<AppData, AppError> {
    if !data.projects.iter().any(|p| p.id == id) {
        return Err(AppError::not_found("project"));
    }
    let mut next = data.clone();
    let task_ids: HashSet<String> = next
        .tasks
        .iter()
        .filter(|t| t.project_id == id)
        .map(|t| t.id.clone())
        .collect();
    remove_tasks(&mut next, &task_ids);
    next.projects.retain(|p| p.id != id);
    Ok(next)
}

// --- tasks -----------------------------------------------------------------

pub fn create_task(
    data: &AppData,
    project_id: &str,
    name: &str,
    clock: &dyn Clock,
) -> Result<AppData, AppError> {
    if !data.projects.iter().any(|p| p.id == project_id) {
        return Err(AppError::no_project());
    }
    let name = clean_name(name)?;
    check_task_name_free(data, project_id, &name, None)?;
    let mut next = data.clone();
    next.tasks.push(Task {
        id: new_id(),
        project_id: project_id.to_string(),
        name,
        created_at: clock.now_ms(),
    });
    Ok(next)
}

pub fn update_task(data: &AppData, id: &str, patch: &TaskPatch) -> Result<AppData, AppError> {
    let task = data
        .tasks
        .iter()
        .find(|t| t.id == id)
        .ok_or_else(|| AppError::not_found("task"))?;
    let name = match &patch.name {
        Some(n) => clean_name(n)?,
        None => task.name.clone(),
    };
    let project_id = match &patch.project_id {
        Some(p) => {
            if !data.projects.iter().any(|x| &x.id == p) {
                return Err(AppError::not_found("project"));
            }
            p.clone()
        }
        None => task.project_id.clone(),
    };
    check_task_name_free(data, &project_id, &name, Some(id))?;
    let mut next = data.clone();
    if let Some(t) = next.tasks.iter_mut().find(|t| t.id == id) {
        t.name = name;
        t.project_id = project_id;
    }
    Ok(next)
}

pub fn delete_task(data: &AppData, id: &str) -> Result<AppData, AppError> {
    if !data.tasks.iter().any(|t| t.id == id) {
        return Err(AppError::not_found("task"));
    }
    let mut next = data.clone();
    let ids: HashSet<String> = std::iter::once(id.to_string()).collect();
    remove_tasks(&mut next, &ids);
    Ok(next)
}

// --- timer -----------------------------------------------------------------

/// Clears the active timer, saving an entry when it ran at least one second.
fn finish_timer(data: &mut AppData, clock: &dyn Clock) {
    if let Some(timer) = data.active_timer.take() {
        let now = clock.now_ms();
        let secs = elapsed_sec(timer.started_at, now);
        if secs >= 1 {
            data.entries.push(Entry {
                id: new_id(),
                task_id: timer.task_id,
                date: clock.local_date(timer.started_at),
                started_at: Some(timer.started_at),
                duration_sec: secs,
                note: String::new(),
                source: Source::Timer,
                created_at: now,
            });
        }
    }
}

/// Starts a timer. A running timer on another task is stopped and saved in the same step.
pub fn start_timer(data: &AppData, task_id: &str, clock: &dyn Clock) -> Result<AppData, AppError> {
    if !data.tasks.iter().any(|t| t.id == task_id) {
        return Err(AppError::not_found("task"));
    }
    if let Some(t) = &data.active_timer {
        if t.task_id == task_id {
            return Ok(data.clone());
        }
    }
    let mut next = data.clone();
    finish_timer(&mut next, clock);
    next.active_timer = Some(ActiveTimer {
        task_id: task_id.to_string(),
        started_at: clock.now_ms(),
    });
    Ok(next)
}

pub fn stop_timer(data: &AppData, clock: &dyn Clock) -> Result<AppData, AppError> {
    if data.active_timer.is_none() {
        return Err(AppError::no_timer());
    }
    let mut next = data.clone();
    finish_timer(&mut next, clock);
    Ok(next)
}

pub fn discard_timer(data: &AppData) -> Result<AppData, AppError> {
    if data.active_timer.is_none() {
        return Err(AppError::no_timer());
    }
    let mut next = data.clone();
    next.active_timer = None;
    Ok(next)
}

// --- entries ---------------------------------------------------------------

pub fn add_entry(data: &AppData, input: &NewEntry, clock: &dyn Clock) -> Result<AppData, AppError> {
    if !data.tasks.iter().any(|t| t.id == input.task_id) {
        return Err(AppError::not_found("task"));
    }
    check_duration(input.duration_sec)?;
    check_date(&input.date, clock)?;
    let note = check_note(&input.note)?;
    let mut next = data.clone();
    next.entries.push(Entry {
        id: new_id(),
        task_id: input.task_id.clone(),
        date: input.date.clone(),
        started_at: None,
        duration_sec: input.duration_sec,
        note,
        source: Source::Manual,
        created_at: clock.now_ms(),
    });
    Ok(next)
}

/// Edits an entry. The duration cap and the future-date rule apply only to values that
/// actually change, so a long timer entry can still have its note or task edited.
/// `started_at` and `source` never change.
pub fn update_entry(
    data: &AppData,
    id: &str,
    patch: &EntryPatch,
    clock: &dyn Clock,
) -> Result<AppData, AppError> {
    let entry = data
        .entries
        .iter()
        .find(|e| e.id == id)
        .ok_or_else(|| AppError::not_found("entry"))?;

    let task_id = match &patch.task_id {
        Some(t) => {
            if !data.tasks.iter().any(|x| &x.id == t) {
                return Err(AppError::not_found("task"));
            }
            t.clone()
        }
        None => entry.task_id.clone(),
    };
    let duration_sec = match patch.duration_sec {
        Some(d) if d != entry.duration_sec => {
            check_duration(d)?;
            d
        }
        Some(d) => d,
        None => entry.duration_sec,
    };
    let date = match &patch.date {
        Some(d) if *d != entry.date => {
            check_date(d, clock)?;
            d.clone()
        }
        Some(d) => d.clone(),
        None => entry.date.clone(),
    };
    let note = match &patch.note {
        Some(n) => check_note(n)?,
        None => entry.note.clone(),
    };

    let mut next = data.clone();
    if let Some(e) = next.entries.iter_mut().find(|e| e.id == id) {
        e.task_id = task_id;
        e.duration_sec = duration_sec;
        e.date = date;
        e.note = note;
    }
    Ok(next)
}

pub fn delete_entry(data: &AppData, id: &str) -> Result<AppData, AppError> {
    if !data.entries.iter().any(|e| e.id == id) {
        return Err(AppError::not_found("entry"));
    }
    let mut next = data.clone();
    next.entries.retain(|e| e.id != id);
    Ok(next)
}

