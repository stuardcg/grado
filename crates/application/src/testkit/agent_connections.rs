//! The agent connections suite and its fake.

use std::collections::HashMap;

use async_trait::async_trait;
use domain::player::PlayerId;

use super::fixtures;
use crate::ports::PortError;
use crate::ports::agent_connections::{AgentConnections, AgentToken};

/// Checks `AgentConnections` built by `make` from the active connections it should hold.
pub async fn suite<A: AgentConnections>(make: impl Fn(Vec<(AgentToken, PlayerId)>) -> A) {
    let player = fixtures::player();
    let token = fixtures::token();
    let second = AgentToken::new("token-ada-laptop").expect("not blank");

    let connections = make(vec![(token.clone(), player.clone()), (second.clone(), player.clone())]);
    assert_eq!(connections.player(&token).await.expect("player"), Some(player.clone()));
    assert_eq!(connections.player(&second).await.expect("player"), Some(player), "a second connection");

    let unknown = AgentToken::new("token-unknown").expect("not blank");
    assert_eq!(connections.player(&unknown).await.expect("player"), None, "unknown token");
}

/// Holds a fixed set of active connections.
pub struct FakeAgentConnections {
    players: HashMap<AgentToken, PlayerId>,
}

impl FakeAgentConnections {
    pub fn new(connections: Vec<(AgentToken, PlayerId)>) -> Self {
        Self { players: connections.into_iter().collect() }
    }
}

#[async_trait]
impl AgentConnections for FakeAgentConnections {
    async fn player(&self, token: &AgentToken) -> Result<Option<PlayerId>, PortError> {
        Ok(self.players.get(token).cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn the_fake_passes_the_suite() {
        suite(FakeAgentConnections::new).await;
    }
}
