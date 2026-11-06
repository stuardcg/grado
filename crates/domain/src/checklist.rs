//! The criteria an agent checks its work against, given only after the work (ADR 0006 #5) and
//! without hints, which are for the app (ADR 0006 #7).

use shared::text::NonBlankString;

use crate::progress::Progress;
use crate::project::{CriterionId, Project, StepId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Checklist {
    /// The open step, the one the report will be about.
    pub step: StepId,
    pub items: Vec<ChecklistItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChecklistItem {
    pub criterion: CriterionId,
    pub title: NonBlankString,
    pub instructions: NonBlankString,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChecklistError {
    /// No step takes a report. `current` is the approved step waiting for its stamp, or `None`
    /// once every step is done or skipped.
    NoOpenStep { current: Option<StepId> },
}

impl Checklist {
    /// `progress` is the player's progress in `project`.
    pub fn new(project: &Project, progress: &Progress) -> Result<Self, ChecklistError> {
        let step = progress
            .open_step()
            .and_then(|id| project.step(id))
            .ok_or_else(|| ChecklistError::NoOpenStep { current: progress.current_step().cloned() })?;
        let items = step
            .criteria
            .iter()
            .map(|criterion| ChecklistItem {
                criterion: criterion.id.clone(),
                title: criterion.title.clone(),
                instructions: criterion.instructions.clone(),
            })
            .collect();
        Ok(Self { step: step.id.clone(), items })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stamp::StepCompletion::{AtApproval, AtStamp};
    use crate::testkit::builders::{criterion_id, step_id, text};
    use crate::testkit::fixtures::{all_pass, bike_registration};

    #[test]
    fn the_open_step_is_listed_without_hints() {
        let project = bike_registration();
        let checklist = Checklist::new(&project, &Progress::start(&project)).expect("an open step");
        assert_eq!(checklist.step, step_id("investigate"));
        assert_eq!(
            checklist.items,
            vec![
                ChecklistItem {
                    criterion: criterion_id("reads-rules"),
                    title: text("reads-rules title"),
                    instructions: text("check reads-rules"),
                },
                ChecklistItem {
                    criterion: criterion_id("lists-fields"),
                    title: text("lists-fields title"),
                    instructions: text("check lists-fields"),
                },
            ]
        );
    }

    #[test]
    fn an_approved_step_takes_no_report() {
        let project = bike_registration();
        let mut progress = Progress::start(&project);
        progress.submit(&project, all_pass(&project, "investigate"), AtStamp).expect("approved");

        assert_eq!(
            Checklist::new(&project, &progress),
            Err(ChecklistError::NoOpenStep { current: Some(step_id("investigate")) })
        );
    }

    #[test]
    fn a_finished_project_has_nothing_to_check() {
        let project = bike_registration();
        let mut progress = Progress::start(&project);
        progress.submit(&project, all_pass(&project, "investigate"), AtApproval).expect("done");
        progress.submit(&project, all_pass(&project, "plan"), AtApproval).expect("done");

        assert_eq!(Checklist::new(&project, &progress), Err(ChecklistError::NoOpenStep { current: None }));
    }
}
