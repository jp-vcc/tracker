use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use serde_json::Value;

use crate::model::{validate_integrity, AppData, SCHEMA_VERSION};

pub const DATA_FILE: &str = "data.json";
pub const MAX_BACKUPS: usize = 5;

const RENAME_ATTEMPTS: u32 = 5;
const RENAME_DELAY: Duration = Duration::from_millis(50);

#[derive(Debug, Clone, PartialEq)]
pub enum LoadOutcome {
    /// File read, parsed and validated.
    Ok(AppData),
    /// No file yet (first launch).
    Empty,
    /// Unreadable content. A backup copy exists at `backup_path`; the original is untouched.
    Corrupt { backup_path: PathBuf, reason: String },
    /// Written by a newer release. Never overwritten except via an explicit reset.
    Newer { version: u32 },
    /// Could not read the file (lock, permissions, not a file ...). Nothing was touched.
    IoError(String),
}

/// Directory that holds the data file and its backups.
pub fn data_dir(path: &Path) -> &Path {
    match path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p,
        _ => Path::new("."),
    }
}

fn tmp_path(path: &Path) -> PathBuf {
    let mut s = path.as_os_str().to_owned();
    s.push(".tmp");
    PathBuf::from(s)
}

/// Runs `op` up to `attempts` times (at least once), sleeping `delay` between
/// failures, and returns the last error if every attempt fails.
pub fn retry<T, E>(
    attempts: u32,
    delay: Duration,
    mut op: impl FnMut() -> Result<T, E>,
) -> Result<T, E> {
    let mut tried = 1;
    loop {
        match op() {
            Ok(v) => return Ok(v),
            Err(e) => {
                if tried >= attempts {
                    return Err(e);
                }
                tried += 1;
                thread::sleep(delay);
            }
        }
    }
}

/// Hook for future schema changes. Identity for version 1.
pub fn migrate(value: Value, from_version: u64) -> Result<Value, String> {
    match from_version {
        1 => Ok(value),
        v => Err(format!("Unsupported schemaVersion {v}.")),
    }
}

enum Parsed {
    Ok(AppData),
    Newer(u32),
    Corrupt(String),
}

fn parse(bytes: &[u8]) -> Parsed {
    let value: Value = match serde_json::from_slice(bytes) {
        Ok(v) => v,
        Err(e) => return Parsed::Corrupt(format!("The file is not valid JSON ({e}).")),
    };
    let version = match value.get("schemaVersion").and_then(Value::as_u64) {
        Some(v) => v,
        None => return Parsed::Corrupt("The file has no schemaVersion.".to_string()),
    };
    if version > u64::from(SCHEMA_VERSION) {
        return Parsed::Newer(u32::try_from(version).unwrap_or(u32::MAX));
    }
    let value = match migrate(value, version) {
        Ok(v) => v,
        Err(m) => return Parsed::Corrupt(m),
    };
    let data: AppData = match serde_json::from_value(value) {
        Ok(d) => d,
        Err(e) => return Parsed::Corrupt(format!("The file has an unexpected shape ({e}).")),
    };
    if let Err(m) = validate_integrity(&data) {
        return Parsed::Corrupt(format!("The data is inconsistent: {m}."));
    }
    Parsed::Ok(data)
}

/// Loads the data file. `now_ms` names the backup if the file turns out to be unreadable.
pub fn load(path: &Path, now_ms: i64) -> LoadOutcome {
    let bytes = match fs::read(path) {
        Ok(b) => b,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return LoadOutcome::Empty,
        Err(e) => {
            return LoadOutcome::IoError(format!("Could not read the data file ({}).", e.kind()))
        }
    };
    match parse(&bytes) {
        Parsed::Ok(d) => LoadOutcome::Ok(d),
        Parsed::Newer(version) => LoadOutcome::Newer { version },
        Parsed::Corrupt(reason) => match backup_corrupt(data_dir(path), &bytes, now_ms) {
            Ok(backup_path) => LoadOutcome::Corrupt {
                backup_path,
                reason,
            },
            Err(_) => LoadOutcome::IoError(
                "Could not back up the unreadable file. Your data file was not changed."
                    .to_string(),
            ),
        },
    }
}

/// Atomically replaces the data file: write tmp, fsync, rename over the target.
pub fn save(path: &Path, data: &AppData) -> io::Result<()> {
    fs::create_dir_all(data_dir(path))?;
    let json = serde_json::to_vec_pretty(data).map_err(io::Error::other)?;
    let tmp = tmp_path(path);
    {
        let mut file = File::create(&tmp)?;
        file.write_all(&json)?;
        file.sync_all()?;
    }
    retry(RENAME_ATTEMPTS, RENAME_DELAY, || fs::rename(&tmp, path))
}

// ---------------------------------------------------------------------------
// Backups
// ---------------------------------------------------------------------------

pub fn backup_corrupt(dir: &Path, bytes: &[u8], ts_ms: i64) -> io::Result<PathBuf> {
    backup(dir, "corrupt", bytes, ts_ms)
}

pub fn backup_newer(dir: &Path, bytes: &[u8], ts_ms: i64) -> io::Result<PathBuf> {
    backup(dir, "newer", bytes, ts_ms)
}

struct Backup {
    ts: i64,
    path: PathBuf,
}

fn backup_name(kind: &str, ts: i64) -> String {
    format!("data.{kind}-{:013}.json", ts.max(0))
}

/// Regular files named `data.<kind>-<digits>.json` in `dir`.
fn list_backups(dir: &Path, kind: &str) -> io::Result<Vec<Backup>> {
    let prefix = format!("data.{kind}-");
    let mut out = Vec::new();
    let entries = match fs::read_dir(dir) {
        Ok(r) => r,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(out),
        Err(e) => return Err(e),
    };
    for item in entries {
        let item = item?;
        let file_name = item.file_name();
        let name = match file_name.to_str() {
            Some(n) => n,
            None => continue,
        };
        let digits = match name
            .strip_prefix(prefix.as_str())
            .and_then(|rest| rest.strip_suffix(".json"))
        {
            Some(d) => d,
            None => continue,
        };
        if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
            continue;
        }
        let ts: i64 = match digits.parse() {
            Ok(t) => t,
            Err(_) => continue,
        };
        if !item.file_type()?.is_file() {
            continue;
        }
        out.push(Backup {
            ts,
            path: item.path(),
        });
    }
    Ok(out)
}

/// One backup rule for both prefixes:
/// 1. reuse an existing backup with identical bytes (nothing written or pruned);
/// 2. otherwise write a new one, bumping the timestamp past same-name clashes;
/// 3. then keep at most `MAX_BACKUPS`, always keeping the new file plus the
///    highest name timestamps among the rest.
fn backup(dir: &Path, kind: &str, bytes: &[u8], ts_ms: i64) -> io::Result<PathBuf> {
    let existing = list_backups(dir, kind)?;
    for b in &existing {
        if fs::metadata(&b.path)?.len() == bytes.len() as u64
            && fs::read(&b.path)?.as_slice() == bytes
        {
            return Ok(b.path.clone());
        }
    }

    fs::create_dir_all(dir)?;
    let mut ts = ts_ms.max(0);
    let new_path = loop {
        let candidate = dir.join(backup_name(kind, ts));
        match fs::symlink_metadata(&candidate) {
            Ok(m) if m.is_file() => ts += 1,
            Ok(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    "backup name is taken by something that is not a file",
                ))
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                match OpenOptions::new().write(true).create_new(true).open(&candidate) {
                    Ok(mut file) => {
                        file.write_all(bytes)?;
                        file.sync_all()?;
                        break candidate;
                    }
                    Err(e) if e.kind() == io::ErrorKind::AlreadyExists => ts += 1,
                    Err(e) => return Err(e),
                }
            }
            Err(e) => return Err(e),
        }
    };

    // Best effort: a failed prune must not turn a good backup into an error.
    if let Ok(all) = list_backups(dir, kind) {
        let mut others: Vec<Backup> = all.into_iter().filter(|b| b.path != new_path).collect();
        others.sort_by(|a, b| b.ts.cmp(&a.ts).then_with(|| b.path.cmp(&a.path)));
        for old in others.into_iter().skip(MAX_BACKUPS - 1) {
            let _ = fs::remove_file(&old.path);
        }
    }
    Ok(new_path)
}

