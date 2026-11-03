//! An agent's report on a step, and what it leads to. `Progress::submit` checks a report and
//! returns a `ReportOutcome` when it is recorded, or a `ReportRefused` when it isn't.

use shared::text::NonBlankString;

use crate::project::{CriterionId, StepId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Pass,
    Fail,
}

/// A short summary the agent sends with an all-pass report, shown in the next briefing. It has no
/// minimum length, but it can't be blank.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandoffNote(NonBlankString);

impl HandoffNote {
    /// Returns `None` for blank text: a blank note counts as no note (ADR 0007).
    pub fn new(text: impl Into<String>) -> Option<Self> {
        NonBlankString::new(text).map(Self)
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// Pass or fail for each of a step's criteria, with no evidence (ADR 0006 #4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub step: StepId,
    pub verdicts: Vec<(CriterionId, Verdict)>,
    pub handoff_note: Option<HandoffNote>,
}

/// What a recorded report did to its step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReportOutcome {
    /// Recorded; the agent stops and tells the person. Listed in the step's criteria order.
    Failed { failed: Vec<CriterionId> },
    /// Every criterion passed and the stamp is on: the step waits for a stamp.
    Approved,
    /// Every criterion passed and the stamp is off: the step is done and `next` is open, if any.
    Done { next: Option<StepId> },
}

/// A report that isn't recorded at all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReportRefused {
    /// The reported step isn't the open one (E1). `open` is the step that is, if any.
    StepNotOpen { open: Option<StepId> },
    /// The verdicts don't cover exactly the step's criteria, once each (E1).
    CriteriaMismatch { expected: Vec<CriterionId> },
    /// Everything passed but there is no handoff note (ADR 0006 #11).
    HandoffNoteMissing,
}

#[cfg(test)]
mod tests {
    use super::HandoffNote;

    #[test]
    fn a_blank_note_is_no_note() {
        assert_eq!(HandoffNote::new("  \n"), None);
    }

    #[test]
    fn a_short_note_is_kept_as_written() {
        assert_eq!(HandoffNote::new("ok").map(|n| n.as_str().to_owned()), Some("ok".to_owned()));
    }
}
