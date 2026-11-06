//! What use cases need from outside the core, one trait per concern (ADR 0004 rule 10).

pub mod agent_connections;
pub mod catalog;
pub mod progress;
pub mod settings;
pub mod stamp_codes;

/// A port couldn't do its work for a reason outside the game, such as storage being unreachable.
/// Test fakes never return it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortError {
    Unavailable,
}
