//! Sample data to run the app on, written in Rust until the content format is decided (STU-27).
//! One project with two steps, one player who started it, and one agent connection, whose token
//! the composition root passes in. Dev only: STU-29 replaces the seeded connection.

use application::AgentToken;
use application::ports::progress::ProgressStore;
use domain::player::PlayerId;
use domain::progress::Progress;
use domain::project::{Criterion, Project, Step};
use domain::settings::GameSettings;
use domain::stamp::StampSetting;
use shared::id::Id;
use shared::text::NonBlankString;

use crate::agent_connections::MemoryAgentConnections;
use crate::catalog::MemoryCatalog;
use crate::progress::MemoryProgress;
use crate::settings::MemorySettings;
use crate::stamp_codes::MemoryStampCodes;

/// One in-memory adapter per port, holding the sample data.
pub struct Sample {
    pub catalog: MemoryCatalog,
    pub progress: MemoryProgress,
    pub settings: MemorySettings,
    pub connections: MemoryAgentConnections,
    pub stamp_codes: MemoryStampCodes,
}

/// The sample project, started by player `ada`, whose one agent connection uses `token`.
pub async fn sample(token: AgentToken, stamp: StampSetting) -> Sample {
    let project = project();
    let player: PlayerId = id("ada");
    let progress = MemoryProgress::new();
    progress
        .save(&player, &Progress::start(&project), None)
        .await
        .expect("an empty store saves new progress");
    Sample {
        catalog: MemoryCatalog::new(vec![project]),
        progress,
        settings: MemorySettings::new(GameSettings { stamp }, Vec::new(), Vec::new()),
        connections: MemoryAgentConnections::new(vec![(token, player)]),
        stamp_codes: MemoryStampCodes::new(),
    }
}

/// A reading list in two steps, each with two criteria.
fn project() -> Project {
    let steps = vec![
        step(
            "books",
            "Keep a list of books",
            "Add books with a title and an author, and keep the list across restarts.",
            [
                criterion(
                    "book-saved",
                    "A book can be added and saved",
                    "Add a book, restart the app and check it is still listed.",
                    "Write the list to a file or a local database, not only to memory.",
                ),
                criterion(
                    "title-required",
                    "A book needs a title",
                    "Try to add a book with an empty title and check it is refused.",
                    "Check the title before saving, and show why it was refused.",
                ),
            ],
        ),
        step(
            "finished",
            "Mark books as read",
            "Mark a book as read and show the books read this year.",
            [
                criterion(
                    "read-marked",
                    "A book can be marked as read",
                    "Mark a book as read, restart the app and check it is still marked.",
                    "Store the date it was finished next to the book.",
                ),
                criterion(
                    "year-filtered",
                    "Only this year's books are shown",
                    "Mark one book read this year and one last year, and check only the first shows.",
                    "Compare the year of each finish date with the current year.",
                ),
            ],
        ),
    ];
    Project::new(id("reading-list"), text("Reading list"), steps).expect("the sample project is valid")
}

fn step(step_id: &str, title: &str, guide: &str, criteria: [Criterion; 2]) -> Step {
    Step { id: id(step_id), title: text(title), guide: text(guide), criteria: criteria.into() }
}

fn criterion(criterion_id: &str, title: &str, instructions: &str, hint: &str) -> Criterion {
    Criterion { id: id(criterion_id), title: text(title), instructions: text(instructions), hint: text(hint) }
}

fn id<T>(value: &str) -> Id<T> {
    Id::new(value).expect("sample ids aren't blank")
}

fn text(value: &str) -> NonBlankString {
    NonBlankString::new(value).expect("sample text isn't blank")
}
