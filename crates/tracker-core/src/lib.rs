//! Tauri-free core of the time tracker: data model, rules, atomic persistence
//! and the application state machine. The `src-tauri` crate is a thin shell
//! around this crate, so everything here can be tested without linking Tauri.

pub mod clock;
pub mod error;
pub mod model;
pub mod service;
pub mod state;
pub mod store;

//! Tauri-free core of the time tracker: data model, rules, atomic persistence
//! and the application state machine. The `src-tauri` crate is a thin shell
//! around this crate, so everything here can be tested without linking Tauri.

pub mod clock;
pub mod error;
pub mod model;
pub mod service;
pub mod state;
pub mod store;

