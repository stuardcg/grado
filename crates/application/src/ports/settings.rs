//! Settings at the game, project and player levels.

use async_trait::async_trait;
use domain::player::PlayerId;
use domain::project::ProjectId;
use domain::settings::{GameSettings, PlayerSettings, ProjectSettings};

use super::PortError;

#[async_trait]
pub trait SettingsStore: Send + Sync {
    /// The three levels that apply to `player` in `project`, for `settings::step_completion`. A
    /// level with nothing stored is its default.
    async fn load(
        &self,
        player: &PlayerId,
        project: &ProjectId,
    ) -> Result<(GameSettings, ProjectSettings, PlayerSettings), PortError>;
}
