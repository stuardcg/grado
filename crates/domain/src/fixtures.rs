//! A small project shared by this crate's tests.

use shared::text::NonBlankString;

use crate::project::{Criterion, CriterionId, Project, ProjectId, Step, StepId};
use crate::report::{HandoffNote, SubmitReport, Verdict};

pub fn step_id(value: &str) -> StepId {
    StepId::new(value).expect("test ids aren't blank")
}

pub fn text(value: &str) -> NonBlankString {
    NonBlankString::new(value).expect("test text isn't blank")
}

/// `investigate` (reads-rules, lists-fields), then `plan` (has-plan, plan-reviewed).
pub fn bike_registration() -> Project {
    let id = ProjectId::new("bike-registration").expect("test ids aren't blank");
    let steps = vec![
        step("investigate", &["reads-rules", "lists-fields"]),
        step("plan", &["has-plan", "plan-reviewed"]),
    ];
    Project::new(id, text("Bike registration"), steps).expect("valid project")
}

/// An all-pass report on `step` of `bike_registration`, with a handoff note.
pub fn all_pass(project: &Project, step: &str) -> SubmitReport {
    let step = project.step(&step_id(step)).expect("a step of the project");
    SubmitReport {
        step: step.id.clone(),
        verdicts: step.criteria.iter().map(|criterion| (criterion.id.clone(), Verdict::Pass)).collect(),
        handoff_note: HandoffNote::new(format!("{} is done", step.id.as_str())),
    }
}

fn step(id: &str, criteria: &[&str]) -> Step {
    Step {
        id: step_id(id),
        title: text(&format!("{id} title")),
        guide: text(&format!("{id} guide")),
        criteria: criteria
            .iter()
            .map(|c| Criterion {
                id: CriterionId::new(*c).expect("test ids aren't blank"),
                title: text(&format!("{c} title")),
                instructions: text(&format!("check {c}")),
                hint: text(&format!("{c} hint")),
            })
            .collect(),
    }
}
