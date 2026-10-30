//! Whether a step needs a stamp, and when one may be issued.

use crate::project::{CriterionId, StepId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StampMode {
    /// The step waits at `approved` until the person enters a stamp.
    On,
    /// The step is done at approval; there is no stamp.
    Off,
}

/// The stamp setting at its three levels (O1). The most specific one that is set wins.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StampSettings {
    pub global: StampMode,
    pub project: Option<StampMode>,
    pub player: Option<StampMode>,
}

impl StampSettings {
    pub fn resolve(&self) -> StampMode {
        self.player.or(self.project).unwrap_or(self.global)
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

#[cfg(test)]
mod tests {
    use super::StampMode::{Off, On};
    use super::*;

    #[test]
    fn global_applies_when_nothing_else_is_set() {
        let settings = StampSettings { global: On, project: None, player: None };
        assert_eq!(settings.resolve(), On);
    }

    #[test]
    fn project_wins_over_global() {
        let settings = StampSettings { global: On, project: Some(Off), player: None };
        assert_eq!(settings.resolve(), Off);
    }

    #[test]
    fn player_wins_over_project() {
        let settings = StampSettings { global: Off, project: Some(Off), player: Some(On) };
        assert_eq!(settings.resolve(), On);
    }
}
