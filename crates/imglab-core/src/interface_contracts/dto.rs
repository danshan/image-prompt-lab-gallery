//! Runtime-facing interface DTO compatibility surface.
//!
//! These types preserve CLI, daemon, Tauri and desktop payload shapes while
//! domain/application code migrates to owned models and ports.

pub use crate::dto::*;
