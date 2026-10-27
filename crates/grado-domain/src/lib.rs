//! Domain rules for Grado.
//!
//! Everything a level *is* (briefing, criteria, guides) lives in the `content/`
//! directory as Markdown and TOML. This crate loads that content, decides what a
//! player may see next, and judges evidence the coaching agent submits. It does
//! not know about HTTP, MCP, or the database.

pub mod briefing;
pub mod catalog;
pub mod checks;
pub mod journey;
pub mod level;
pub mod secrets;
mod template;

pub use template::Vars;

pub use catalog::{Catalog, CatalogError};
pub use checks::{OfflineRubricGrader, RubricGrader, RubricRequest, Verdict, evaluate};
pub use journey::{Journey, LevelStatus};
pub use level::{Check, Criterion, Environment, EvidenceKind, Guide, Level};
