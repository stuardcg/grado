//! The connections that let a player's LLM client act for them.

use std::fmt;

use async_trait::async_trait;
use domain::player::PlayerId;
use shared::text::NonBlankString;

use super::PortError;

/// The secret an agent sends to identify its connection. Its value never appears in `Debug`.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct AgentToken(NonBlankString);

impl AgentToken {
    /// Returns `None` when `token` is empty or only whitespace.
    pub fn new(token: impl Into<String>) -> Option<Self> {
        NonBlankString::new(token).map(Self)
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Debug for AgentToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("AgentToken(..)")
    }
}

#[async_trait]
pub trait AgentConnections: Send + Sync {
    /// The player whose active connection `token` belongs to. `None` for an unknown or revoked
    /// token.
    async fn player(&self, token: &AgentToken) -> Result<Option<PlayerId>, PortError>;
}
