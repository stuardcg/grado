//! Settings at each level: the game, a project and a player. A project or player setting that
//! isn't set follows the level above it.

use crate::stamp::{StampSetting, StepCompletion};

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

/// When a player's steps in a project complete, from the stamp setting: the player's, else the
/// project's, else the game's (O1).
pub fn step_completion(
    game: &GameSettings,
    project: &ProjectSettings,
    player: &PlayerSettings,
) -> StepCompletion {
    match player.stamp.or(project.stamp).unwrap_or(game.stamp) {
        StampSetting::On => StepCompletion::AtStamp,
        StampSetting::Off => StepCompletion::AtApproval,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stamp::StampSetting::{Off, On};
    use crate::stamp::StepCompletion::{AtApproval, AtStamp};

    #[test]
    fn the_game_applies_when_nothing_else_is_set() {
        let completion = step_completion(
            &GameSettings { stamp: On },
            &ProjectSettings::default(),
            &PlayerSettings::default(),
        );
        assert_eq!(completion, AtStamp);
    }

    #[test]
    fn the_project_wins_over_the_game() {
        let completion = step_completion(
            &GameSettings { stamp: On },
            &ProjectSettings { stamp: Some(Off) },
            &PlayerSettings::default(),
        );
        assert_eq!(completion, AtApproval);
    }

    #[test]
    fn the_player_wins_over_the_project() {
        let completion = step_completion(
            &GameSettings { stamp: Off },
            &ProjectSettings { stamp: Some(Off) },
            &PlayerSettings { stamp: Some(On) },
        );
        assert_eq!(completion, AtStamp);
    }
}
