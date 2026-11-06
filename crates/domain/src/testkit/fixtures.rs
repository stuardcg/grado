//! Example data for this crate's tests.

use super::builders::{step, step_id, text};
use crate::project::{Project, ProjectId};
use crate::report::{HandoffNote, SubmitReport, Verdict};

/// `investigate` (reads-rules, lists-fields), then `plan` (has-plan, plan-reviewed).
pub fn bike_registration() -> Project {
    let id = ProjectId::new("bike-registration").expect("test ids aren't blank");
    let steps = vec![
        step("investigate", &["reads-rules", "lists-fields"]),
        step("plan", &["has-plan", "plan-reviewed"]),
    ];
    Project::new(id, text("Bike registration"), steps).expect("valid project")
}

/// An all-pass report on `step` of `project`, with the note `<step> is done`.
pub fn all_pass(project: &Project, step: &str) -> SubmitReport {
    let step = project.step(&step_id(step)).expect("a step of the project");
    SubmitReport {
        step: step.id.clone(),
        verdicts: step.criteria.iter().map(|criterion| (criterion.id.clone(), Verdict::Pass)).collect(),
        handoff_note: HandoffNote::new(format!("{} is done", step.id.as_str())),
    }
}
