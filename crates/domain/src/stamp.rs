//! Whether a step needs a stamp, and when one may be issued.

use crate::project::{CriterionId, StepId};

/// The stamp setting stored at one level: the game, a project or a player.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StampSetting {
    /// The step waits at `approved` until the person enters a stamp.
    On,
    /// The step is done at approval; there is no stamp.
    Off,
}

/// The stamp setting that applies, once the levels are resolved (O1). Outside this crate, the
/// only way to get one is `settings::stamp_mode`, so no caller can skip the levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StampMode(StampSetting);

impl StampMode {
    pub(crate) const fn new(setting: StampSetting) -> Self {
        Self(setting)
    }

    pub fn is_on(self) -> bool {
        self.0 == StampSetting::On
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StampRefused {
    /// The step isn't approved yet (E2). `pending` lists the criteria that haven't passed in its
    /// latest report, or all of them when there is no report.
    NotApproved { pending: Vec<CriterionId> },
    /// The stamp is off, so the step was done at approval (E14).
    StampOff,
    /// The project has no such step.
    UnknownStep(StepId),
}
