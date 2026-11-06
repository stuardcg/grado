//! Short constructors for test data. Text is derived from the id, so `step("plan", ..)` has the
//! title `plan title` and the guide `plan guide`.

use shared::text::NonBlankString;

use crate::project::{Criterion, CriterionId, Step, StepId};
use crate::report::{HandoffNote, SubmitReport, Verdict};

pub fn step_id(value: &str) -> StepId {
    StepId::new(value).expect("test ids aren't blank")
}

pub fn criterion_id(value: &str) -> CriterionId {
    CriterionId::new(value).expect("test ids aren't blank")
}

pub fn text(value: &str) -> NonBlankString {
    NonBlankString::new(value).expect("test text isn't blank")
}

pub fn criterion(id: &str) -> Criterion {
    Criterion {
        id: criterion_id(id),
        title: text(&format!("{id} title")),
        instructions: text(&format!("check {id}")),
        hint: text(&format!("{id} hint")),
    }
}

pub fn step(id: &str, criteria: &[&str]) -> Step {
    Step {
        id: step_id(id),
        title: text(&format!("{id} title")),
        guide: text(&format!("{id} guide")),
        criteria: criteria.iter().map(|c| criterion(c)).collect(),
    }
}

/// A report on `step`. A blank `note` gives no note, as `HandoffNote::new` does.
pub fn report(step: &str, verdicts: &[(&str, Verdict)], note: Option<&str>) -> SubmitReport {
    SubmitReport {
        step: step_id(step),
        verdicts: verdicts.iter().map(|(c, v)| (criterion_id(c), *v)).collect(),
        handoff_note: note.and_then(HandoffNote::new),
    }
}
