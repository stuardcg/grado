//! What use cases need from outside the core, one trait per concern (ADR 0004 rule 10).

pub mod agent_connections;
pub mod catalog;
pub mod progress;
pub mod settings;
pub mod stamp_codes;

use std::error::Error;
use std::fmt;
use std::sync::Arc;

/// The cause of a port failure, as the outbound adapter reported it.
pub type Cause = Arc<dyn Error + Send + Sync>;

/// A port couldn't do its work for a reason outside the game, such as storage being unreachable.
/// Test fakes never return it. The cause is only for the log (ADR 0012), so two port errors are
/// equal when their kind is.
#[derive(Debug, Clone)]
pub enum PortError {
    Unavailable(Cause),
}

impl PortError {
    pub fn unavailable(cause: impl Into<Box<dyn Error + Send + Sync>>) -> Self {
        Self::Unavailable(Arc::from(cause.into()))
    }
}

impl PartialEq for PortError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Unavailable(_), Self::Unavailable(_)) => true,
        }
    }
}

impl Eq for PortError {}

impl fmt::Display for PortError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable(cause) => write!(f, "unavailable: {cause}"),
        }
    }
}

impl Error for PortError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Unavailable(cause) => Some(&**cause),
        }
    }
}
