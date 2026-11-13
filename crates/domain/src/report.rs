//! An agent's report on a step, and what it leads to. `Progress::submit` checks a report and
//! returns a `Submitted` when it is recorded, or a `SubmitError` when it isn't.

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

/// The report an agent sends: pass or fail for each of a step's criteria, with no evidence
/// (ADR 0006 #4). Nothing about it is checked until `Progress::submit`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubmitReport {
    pub step: StepId,
    pub verdicts: Vec<(CriterionId, Verdict)>,
    pub handoff_note: Option<HandoffNote>,
}

/// A report `Progress::submit` accepted, with the answer it got. Its verdicts cover exactly the
/// step's criteria, once each.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepReport {
    step: StepId,
    verdicts: Vec<(CriterionId, Verdict)>,
    handoff_note: Option<HandoffNote>,
    answer: Submitted,
}

impl StepReport {
    pub(crate) fn accept(report: SubmitReport, answer: Submitted) -> Self {
        Self { step: report.step, verdicts: report.verdicts, handoff_note: report.handoff_note, answer }
    }

    /// Whether `report` repeats this one (E4): the same step, the same verdicts in any order, and
    /// the same handoff note.
    pub(crate) fn is_repeated_by(&self, report: &SubmitReport) -> bool {
        self.step == report.step
            && self.handoff_note == report.handoff_note
            && self.verdicts.len() == report.verdicts.len()
            && self.verdicts.iter().all(|verdict| report.verdicts.contains(verdict))
    }

    pub fn step(&self) -> &StepId {
        &self.step
    }

    pub fn verdicts(&self) -> &[(CriterionId, Verdict)] {
        &self.verdicts
    }

    pub fn handoff_note(&self) -> Option<&HandoffNote> {
        self.handoff_note.as_ref()
    }

    /// The answer `Progress::submit` gave this report.
    pub fn answer(&self) -> &Submitted {
        &self.answer
    }
}

/// What an accepted report did to its step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Submitted {
    /// Some criteria failed; the agent stops and tells the person. Listed in the step's criteria
    /// order.
    StepStillOpen { failed: Vec<CriterionId> },
    /// Every criterion passed and the step completes at its stamp: it waits for one.
    StepApproved,
    /// Every criterion passed and the step completes at approval: it is done and `next` is open,
    /// if any.
    StepDone { next: Option<StepId> },
}

/// Why a report wasn't recorded at all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubmitError {
    /// The project has no such step.
    UnknownStep(StepId),
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
