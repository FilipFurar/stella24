//! # Stella24
//! ER modeler for Oracle SQL

/// # Main application module
/// Connects model and UI
pub mod app;

/// # Model module
/// Contains all backend logic
pub mod model;

/// # UI module
/// Contains UI logic and drawing
pub mod ui;

pub use app::AppStella;
