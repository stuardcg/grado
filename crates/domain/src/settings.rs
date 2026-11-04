//! Settings at each level: the game, a project and a player. A project or player setting that
//! isn't set follows the level above it.

use crate::stamp::{StampMode, StampSetting};

/// Settings for the whole game. Every one has a value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameSettings {
    pub stamp: StampSetting,
}

/// Settings for one project, kept apart from its content.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProjectSettings {
    pub stamp: Option<StampSetting>,
}

/// Settings for one player, across all their projects.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PlayerSettings {
    pub stamp: Option<StampSetting>,
}

/// The stamp mode for a player in a project: the player's setting, else the project's, else the
/// game's (O1).
pub fn stamp_mode(game: &GameSettings, project: &ProjectSettings, player: &PlayerSettings) -> StampMode {
    StampMode::new(player.stamp.or(project.stamp).unwrap_or(game.stamp))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stamp::StampSetting::{Off, On};

    #[test]
    fn the_game_applies_when_nothing_else_is_set() {
        let mode =
            stamp_mode(&GameSettings { stamp: On }, &ProjectSettings::default(), &PlayerSettings::default());
        assert!(mode.is_on());
    }

    #[test]
    fn the_project_wins_over_the_game() {
        let mode = stamp_mode(
            &GameSettings { stamp: On },
            &ProjectSettings { stamp: Some(Off) },
            &PlayerSettings::default(),
        );
        assert!(!mode.is_on());
    }

    #[test]
    fn the_player_wins_over_the_project() {
        let mode = stamp_mode(
            &GameSettings { stamp: Off },
            &ProjectSettings { stamp: Some(Off) },
            &PlayerSettings { stamp: Some(On) },
        );
        assert!(mode.is_on());
    }
}
