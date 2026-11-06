//! The settings suite and its fake.

use std::collections::HashMap;

use async_trait::async_trait;
use domain::player::PlayerId;
use domain::project::ProjectId;
use domain::settings::{GameSettings, PlayerSettings, ProjectSettings};
use domain::stamp::StampSetting;

use super::fixtures;
use crate::ports::PortError;
use crate::ports::settings::SettingsStore;

/// What a settings store holds at each level.
#[derive(Debug, Clone)]
pub struct StoredSettings {
    pub game: GameSettings,
    pub projects: Vec<(ProjectId, ProjectSettings)>,
    pub players: Vec<(PlayerId, PlayerSettings)>,
}

impl StoredSettings {
    /// Only the game level is set.
    pub fn game(game: GameSettings) -> Self {
        Self { game, projects: Vec::new(), players: Vec::new() }
    }
}

/// Checks a `SettingsStore` built by `make` from what it should hold.
pub async fn suite<S: SettingsStore>(make: impl Fn(StoredSettings) -> S) {
    let player = fixtures::player();
    let project = fixtures::project().id().clone();

    let store = make(StoredSettings::game(fixtures::stamp_on()));
    assert_eq!(
        store.load(&player, &project).await.expect("load"),
        (fixtures::stamp_on(), ProjectSettings::default(), PlayerSettings::default()),
        "unset levels are their default",
    );

    let project_level = ProjectSettings { stamp: Some(StampSetting::Off) };
    let player_level = PlayerSettings { stamp: Some(StampSetting::On) };
    let store = make(StoredSettings {
        game: fixtures::stamp_off(),
        projects: vec![(project.clone(), project_level.clone())],
        players: vec![(player.clone(), player_level.clone())],
    });
    assert_eq!(
        store.load(&player, &project).await.expect("load"),
        (fixtures::stamp_off(), project_level, player_level),
        "every stored level",
    );

    let other_player = PlayerId::new("grace").expect("not blank");
    let other_project = ProjectId::new("other-project").expect("not blank");
    assert_eq!(
        store.load(&other_player, &other_project).await.expect("load"),
        (fixtures::stamp_off(), ProjectSettings::default(), PlayerSettings::default()),
        "levels belong to their own project and player",
    );
}

/// Holds fixed settings.
pub struct FakeSettings {
    game: GameSettings,
    projects: HashMap<ProjectId, ProjectSettings>,
    players: HashMap<PlayerId, PlayerSettings>,
}

impl FakeSettings {
    pub fn new(stored: StoredSettings) -> Self {
        Self {
            game: stored.game,
            projects: stored.projects.into_iter().collect(),
            players: stored.players.into_iter().collect(),
        }
    }
}

#[async_trait]
impl SettingsStore for FakeSettings {
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
    use super::*;

    #[tokio::test]
    async fn the_fake_passes_the_suite() {
        suite(FakeSettings::new).await;
    }
}
