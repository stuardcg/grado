//! Whether a step needs a stamp, and when one may be issued.

use shared::text::NonBlankString;

use crate::project::{CriterionId, StepId};

/// The stamp setting stored at one level: the game, a project or a player.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StampSetting {
    /// The step waits at `approved` until the person enters a stamp.
    On,
    /// The step is done at approval; there is no stamp.
    Off,
}

/// When a step completes, once the stamp setting's levels are resolved by
/// `settings::step_completion` (O1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepCompletion {
    /// The stamp is on: the step waits at `approved` for its stamp.
    AtStamp,
    /// The stamp is off: the step is done at approval.
    AtApproval,
}

/// Why a stamp may not be issued.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StampError {
    /// The step isn't approved yet (E2). `pending` lists the criteria that haven't passed in its
    /// latest report, or all of them when there is no report.
    NotApproved { pending: Vec<CriterionId> },
    /// The stamp is off, so the step was done at approval (E14).
    StampOff,
    /// The project has no such step.
    UnknownStep(StepId),
}

/// The code a person enters to complete an approved step when the stamp is on.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StampCode(NonBlankString);

impl StampCode {
    /// Returns `None` when `code` is empty or only whitespace.
    pub fn new(code: impl Into<String>) -> Option<Self> {
        NonBlankString::new(code).map(Self)
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}
