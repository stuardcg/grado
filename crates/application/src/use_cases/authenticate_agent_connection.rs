//! Finds the player an agent acts for, from its connection's token (E8). Every other MCP call
//! runs after this one, with the `PlayerId` it returns (ADR 0004 rule 24).

use domain::player::PlayerId;

use crate::ports::PortError;
use crate::ports::agent_connections::{AgentConnections, AgentToken};

#[derive(Debug, Clone)]
pub struct Input {
    pub token: AgentToken,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output {
    pub player: PlayerId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The token is unknown or its connection was revoked.
    NoActiveConnection,
    Port(PortError),
}

pub async fn run(connections: &dyn AgentConnections, input: Input) -> Result<Output, Error> {
    let player =
        connections.player(&input.token).await.map_err(Error::Port)?.ok_or(Error::NoActiveConnection)?;
    Ok(Output { player })
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;

    use super::*;
    use crate::testkit::agent_connections::FakeAgentConnections;
    use crate::testkit::fixtures;

    fn connections() -> FakeAgentConnections {
        FakeAgentConnections::new(vec![(fixtures::token(), fixtures::player())])
    }

    #[tokio::test]
    async fn an_active_connection_gives_its_player() {
        let output = run(&connections(), Input { token: fixtures::token() }).await;
        assert_eq!(output, Ok(Output { player: fixtures::player() }));
    }

    #[tokio::test]
    async fn an_unknown_or_revoked_token_is_refused() {
        let token = AgentToken::new("token-revoked").expect("not blank");
        assert_eq!(run(&connections(), Input { token }).await, Err(Error::NoActiveConnection));
    }

    #[tokio::test]
    async fn a_port_failure_is_passed_on() {
        struct Down;

        #[async_trait]
        impl AgentConnections for Down {
            async fn player(&self, _: &AgentToken) -> Result<Option<PlayerId>, PortError> {
                Err(PortError::Unavailable)
            }
        }

        let output = run(&Down, Input { token: fixtures::token() }).await;
        assert_eq!(output, Err(Error::Port(PortError::Unavailable)));
    }
}
