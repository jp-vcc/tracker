use std::fmt;

use serde::{Deserialize, Serialize};

pub const VALIDATION: &str = "validation";
pub const DUPLICATE: &str = "duplicate";
pub const NOT_FOUND: &str = "not_found";
pub const NO_PROJECT: &str = "no_project";
pub const NO_TIMER: &str = "no_timer";
pub const RECOVERY_REQUIRED: &str = "recovery_required";
pub const INVALID_STATE: &str = "invalid_state";
pub const PERSISTENCE: &str = "persistence";

/// Error returned to the UI. `field` lets the UI place the message next to the
/// right input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppError {
    pub code: String,
    pub field: Option<String>,
    pub message: String,
}

impl AppError {
    fn new(code: &str, field: Option<&str>, message: impl Into<String>) -> Self {
        AppError {
            code: code.to_string(),
            field: field.map(str::to_string),
            message: message.into(),
        }
    }

    pub fn validation(field: &str, message: impl Into<String>) -> Self {
        Self::new(VALIDATION, Some(field), message)
    }

    pub fn duplicate(field: &str, message: impl Into<String>) -> Self {
        Self::new(DUPLICATE, Some(field), message)
    }

    pub fn not_found(what: &str) -> Self {
        Self::new(NOT_FOUND, None, format!("The {what} no longer exists."))
    }

    pub fn no_project() -> Self {
        Self::new(NO_PROJECT, None, "Create a project before adding tasks.")
    }

    pub fn no_timer() -> Self {
        Self::new(NO_TIMER, None, "No timer is running.")
    }

    pub fn recovery_required() -> Self {
        Self::new(
            RECOVERY_REQUIRED,
            None,
            "Your data file needs attention before changes can be saved.",
        )
    }

    pub fn invalid_state(message: impl Into<String>) -> Self {
        Self::new(INVALID_STATE, None, message)
    }

    pub fn persistence(message: impl Into<String>) -> Self {
        Self::new(PERSISTENCE, None, message)
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for AppError {}

use std::fmt;

use serde::{Deserialize, Serialize};

pub const VALIDATION: &str = "validation";
pub const DUPLICATE: &str = "duplicate";
pub const NOT_FOUND: &str = "not_found";
pub const NO_PROJECT: &str = "no_project";
pub const NO_TIMER: &str = "no_timer";
pub const RECOVERY_REQUIRED: &str = "recovery_required";
pub const INVALID_STATE: &str = "invalid_state";
pub const PERSISTENCE: &str = "persistence";

/// Error returned to the UI. `field` lets the UI place the message next to the
/// right input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppError {
    pub code: String,
    pub field: Option<String>,
    pub message: String,
}

impl AppError {
    fn new(code: &str, field: Option<&str>, message: impl Into<String>) -> Self {
        AppError {
            code: code.to_string(),
            field: field.map(str::to_string),
            message: message.into(),
        }
    }

    pub fn validation(field: &str, message: impl Into<String>) -> Self {
        Self::new(VALIDATION, Some(field), message)
    }

    pub fn duplicate(field: &str, message: impl Into<String>) -> Self {
        Self::new(DUPLICATE, Some(field), message)
    }

    pub fn not_found(what: &str) -> Self {
        Self::new(NOT_FOUND, None, format!("The {what} no longer exists."))
    }

    pub fn no_project() -> Self {
        Self::new(NO_PROJECT, None, "Create a project before adding tasks.")
    }

    pub fn no_timer() -> Self {
        Self::new(NO_TIMER, None, "No timer is running.")
    }

    pub fn recovery_required() -> Self {
        Self::new(
            RECOVERY_REQUIRED,
            None,
            "Your data file needs attention before changes can be saved.",
        )
    }

    pub fn invalid_state(message: impl Into<String>) -> Self {
        Self::new(INVALID_STATE, None, message)
    }

    pub fn persistence(message: impl Into<String>) -> Self {
        Self::new(PERSISTENCE, None, message)
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for AppError {}

use std::fmt;

use serde::{Deserialize, Serialize};

pub const VALIDATION: &str = "validation";
pub const DUPLICATE: &str = "duplicate";
pub const NOT_FOUND: &str = "not_found";
pub const NO_PROJECT: &str = "no_project";
pub const NO_TIMER: &str = "no_timer";
pub const RECOVERY_REQUIRED: &str = "recovery_required";
pub const INVALID_STATE: &str = "invalid_state";
pub const PERSISTENCE: &str = "persistence";

/// Error returned to the UI. `field` lets the UI place the message next to the
/// right input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppError {
    pub code: String,
    pub field: Option<String>,
    pub message: String,
}

impl AppError {
    fn new(code: &str, field: Option<&str>, message: impl Into<String>) -> Self {
        AppError {
            code: code.to_string(),
            field: field.map(str::to_string),
            message: message.into(),
        }
    }

    pub fn validation(field: &str, message: impl Into<String>) -> Self {
        Self::new(VALIDATION, Some(field), message)
    }

    pub fn duplicate(field: &str, message: impl Into<String>) -> Self {
        Self::new(DUPLICATE, Some(field), message)
    }

    pub fn not_found(what: &str) -> Self {
        Self::new(NOT_FOUND, None, format!("The {what} no longer exists."))
    }

    pub fn no_project() -> Self {
        Self::new(NO_PROJECT, None, "Create a project before adding tasks.")
    }

    pub fn no_timer() -> Self {
        Self::new(NO_TIMER, None, "No timer is running.")
    }

    pub fn recovery_required() -> Self {
        Self::new(
            RECOVERY_REQUIRED,
            None,
            "Your data file needs attention before changes can be saved.",
        )
    }

    pub fn invalid_state(message: impl Into<String>) -> Self {
        Self::new(INVALID_STATE, None, message)
    }

    pub fn persistence(message: impl Into<String>) -> Self {
        Self::new(PERSISTENCE, None, message)
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for AppError {}

