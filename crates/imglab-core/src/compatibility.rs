//! Legacy root-level exports kept for downstream migration.
//!
//! New code should import through `domain`, `application`, `infrastructure`,
//! `interface_contracts`, or the concrete module that owns the behavior.
//! This module intentionally remains narrow and explicit so compatibility use
//! can be audited during future cleanup.

pub use crate::dto::*;
pub use crate::library::*;
pub use crate::provider::*;
pub use crate::services::*;
pub use crate::task_scheduler::*;
