//! Settings held in memory.

use std::collections::HashMap;

use application::ports::PortError;
use application::ports::settings::SettingsStore;
use async_trait::async_trait;
use domain::player::PlayerId;
use domain::project::ProjectId;
use domain::settings::{GameSettings, PlayerSettings, ProjectSettings};

pub struct MemorySettings {
    game: GameSettings,
    projects: HashMap<ProjectId, ProjectSettings>,
    players: HashMap<PlayerId, PlayerSettings>,
}

impl MemorySettings {
    pub fn new(
        game: GameSettings,
        projects: Vec<(ProjectId, ProjectSettings)>,
        players: Vec<(PlayerId, PlayerSettings)>,
    ) -> Self {
        Self { game, projects: projects.into_iter().collect(), players: players.into_iter().collect() }
    }
}

#[async_trait]
impl SettingsStore for MemorySettings {
    async fn load(
        &self,
        player: &PlayerId,
        project: &ProjectId,
    ) -> Result<(GameSettings, ProjectSettings, PlayerSettings), PortError> {
        Ok((
            self.game.clone(),
            self.projects.get(project).cloned().unwrap_or_default(),
            self.players.get(player).cloned().unwrap_or_default(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use application::testkit::settings::suite;

    use super::MemorySettings;

    #[tokio::test]
    async fn passes_the_settings_suite() {
        suite(|stored| MemorySettings::new(stored.game, stored.projects, stored.players)).await;
    }
}
