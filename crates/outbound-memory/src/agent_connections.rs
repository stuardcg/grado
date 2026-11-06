//! Active agent connections held in memory.

use std::collections::HashMap;

use application::ports::PortError;
use application::ports::agent_connections::{AgentConnections, AgentToken};
use async_trait::async_trait;
use domain::player::PlayerId;

pub struct MemoryAgentConnections {
    players: HashMap<AgentToken, PlayerId>,
}

impl MemoryAgentConnections {
    pub fn new(connections: Vec<(AgentToken, PlayerId)>) -> Self {
        Self { players: connections.into_iter().collect() }
    }
}

#[async_trait]
impl AgentConnections for MemoryAgentConnections {
    async fn player(&self, token: &AgentToken) -> Result<Option<PlayerId>, PortError> {
        Ok(self.players.get(token).cloned())
    }
}

#[cfg(test)]
mod tests {
    use application::testkit::agent_connections::suite;

    use super::MemoryAgentConnections;

    #[tokio::test]
    async fn passes_the_agent_connections_suite() {
        suite(MemoryAgentConnections::new).await;
    }
}
