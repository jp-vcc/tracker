use std::sync::atomic::{AtomicI64, Ordering};

use chrono::{DateTime, Local, TimeZone, Utc};

/// Source of time for the rules. Injectable so tests can fix time and zone.
pub trait Clock {
    /// UTC epoch milliseconds.
    fn now_ms(&self) -> i64;

    /// Local calendar date (`YYYY-MM-DD`) of the given instant.
    fn local_date(&self, ms: i64) -> String;

    /// Local calendar date of "now".
    fn today(&self) -> String {
        self.local_date(self.now_ms())
    }
}

/// Production clock: system time and the system time zone.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_ms(&self) -> i64 {
        Utc::now().timestamp_millis()
    }

    fn local_date(&self, ms: i64) -> String {
        match Local.timestamp_millis_opt(ms).single() {
            Some(dt) => dt.format("%Y-%m-%d").to_string(),
            None => format_with_offset(ms, 0),
        }
    }
}

fn format_with_offset(ms: i64, offset_minutes: i32) -> String {
    DateTime::from_timestamp_millis(ms + i64::from(offset_minutes) * 60_000)
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "1970-01-01".to_string())
}

/// Clock with a settable instant and a fixed UTC offset, for tests.
pub struct FixedClock {
    now: AtomicI64,
    offset_minutes: i32,
}

impl FixedClock {
    pub fn new(now_ms: i64, offset_minutes: i32) -> Self {
        FixedClock {
            now: AtomicI64::new(now_ms),
            offset_minutes,
        }
    }

    pub fn set(&self, now_ms: i64) {
        self.now.store(now_ms, Ordering::SeqCst);
    }

    pub fn advance(&self, delta_ms: i64) {
        self.now.fetch_add(delta_ms, Ordering::SeqCst);
    }
}

impl Clock for FixedClock {
    fn now_ms(&self) -> i64 {
        self.now.load(Ordering::SeqCst)
    }

    fn local_date(&self, ms: i64) -> String {
        format_with_offset(ms, self.offset_minutes)
    }
}

