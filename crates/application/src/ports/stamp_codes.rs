//! Stamp codes, which only a port may generate (ADR 0004 rule 12).

use async_trait::async_trait;
use domain::player::PlayerId;
use domain::project::{ProjectId, StepId};
use domain::stamp::StampCode;

use super::PortError;

#[async_trait]
pub trait StampCodes: Send + Sync {
    /// Generates a new code for `step` and stores it in place of the previous one, which is void
    /// from then on (E10). Codes don't expire.
    async fn issue(
        &self,
        player: &PlayerId,
        project: &ProjectId,
        step: &StepId,
    ) -> Result<StampCode, PortError>;

    /// The code last issued for `step`, the only one still valid. `None` before the first.
    async fn current(
        &self,
        player: &PlayerId,
        project: &ProjectId,
        step: &StepId,
    ) -> Result<Option<StampCode>, PortError>;
}
