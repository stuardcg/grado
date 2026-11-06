//! Test data, written in Rust until the content format is decided (STU-27): one project with two
//! steps, one player who started it, one agent connection, and the stamp on or off.

use domain::player::PlayerId;
use domain::progress::Progress;
use domain::project::{Criterion, Project, Step, StepId};
use domain::settings::GameSettings;
use domain::stamp::StampSetting;
use shared::id::Id;
use shared::text::NonBlankString;

use crate::ports::agent_connections::AgentToken;

/// A habit tracker in two steps, each with two criteria.
pub fn project() -> Project {
    let steps = vec![
        step(
            "data-model",
            "Store habits",
            "Model a habit with a name and the days it was done, and keep it across restarts.",
            [
                criterion(
                    "habit-saved",
                    "A habit can be created and saved",
                    "Create a habit, restart the app and check it is still listed.",
                    "Write habits to a file or a local database, not only to memory.",
                ),
                criterion(
                    "habit-named",
                    "A habit needs a name",
                    "Try to create a habit with an empty name and check it is refused.",
                    "Check the name before saving, and show why it was refused.",
                ),
            ],
        ),
        step(
            "streaks",
            "Count streaks",
            "Mark a habit done for today and show how many days in a row it has been done.",
            [
                criterion(
                    "streak-counted",
                    "The streak counts consecutive days",
                    "Mark a habit done on three days in a row and check the streak shows 3.",
                    "Walk back from today while each previous day is marked done.",
                ),
                criterion(
                    "streak-reset",
                    "A missed day resets the streak",
                    "Skip a day, mark the habit done again and check the streak shows 1.",
                    "A gap of one day ends the streak; start counting again from today.",
                ),
            ],
        ),
    ];
    Project::new(id("habit-tracker"), text("Habit tracker"), steps).expect("the fixture project is valid")
}

pub fn first_step() -> StepId {
    id("data-model")
}

pub fn second_step() -> StepId {
    id("streaks")
}

pub fn player() -> PlayerId {
    id("ada")
}

/// The token of the player's one agent connection.
pub fn token() -> AgentToken {
    AgentToken::new("token-ada").expect("the fixture token isn't blank")
}

/// The player's progress right after starting the project.
pub fn started() -> Progress {
    Progress::start(&project())
}

pub fn stamp_on() -> GameSettings {
    GameSettings { stamp: StampSetting::On }
}

pub fn stamp_off() -> GameSettings {
    GameSettings { stamp: StampSetting::Off }
}

fn step(step_id: &str, title: &str, guide: &str, criteria: [Criterion; 2]) -> Step {
    Step { id: id(step_id), title: text(title), guide: text(guide), criteria: criteria.into() }
}

fn criterion(criterion_id: &str, title: &str, instructions: &str, hint: &str) -> Criterion {
    Criterion { id: id(criterion_id), title: text(title), instructions: text(instructions), hint: text(hint) }
}

fn id<T>(value: &str) -> Id<T> {
    Id::new(value).expect("fixture ids aren't blank")
}

fn text(value: &str) -> NonBlankString {
    NonBlankString::new(value).expect("fixture text isn't blank")
}
