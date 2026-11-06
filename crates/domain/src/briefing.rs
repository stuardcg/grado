//! What a fresh agent gets before it works: the project, where the player is, and the last
//! handoff note. Never the criteria (ADR 0006 #5).

use shared::text::NonBlankString;

use crate::progress::{Progress, StepState};
use crate::project::{Project, ProjectId, StepId};
use crate::report::HandoffNote;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Briefing {
    pub project: ProjectId,
    pub title: NonBlankString,
    /// `None` once every step is done or skipped.
    pub current: Option<CurrentStep>,
    /// The step after the current one, previewed by its title.
    pub next: Option<NextStep>,
    pub handoff_note: Option<HandoffNote>,
}

/// The step being worked on: open, or approved and waiting for its stamp.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrentStep {
    pub id: StepId,
    pub title: NonBlankString,
    pub guide: NonBlankString,
    pub state: StepState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NextStep {
    pub id: StepId,
    pub title: NonBlankString,
}

impl Briefing {
    /// `progress` is the player's progress in `project`.
    pub fn new(project: &Project, progress: &Progress) -> Self {
        let current = progress.current_step().and_then(|id| project.step(id)).and_then(|step| {
            progress.state(&step.id).map(|state| CurrentStep {
                id: step.id.clone(),
                title: step.title.clone(),
                guide: step.guide.clone(),
                state,
            })
        });
        let next = progress
            .next_step()
            .and_then(|id| project.step(id))
            .map(|step| NextStep { id: step.id.clone(), title: step.title.clone() });
        Self {
            project: project.id().clone(),
            title: project.title().clone(),
            current,
            next,
            handoff_note: progress.last_handoff_note().cloned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::{all_pass, bike_registration, step_id, text};
    use crate::stamp::StepCompletion::{AtApproval, AtStamp};

    #[test]
    fn a_new_player_is_briefed_on_the_first_step() {
        let project = bike_registration();
        let briefing = Briefing::new(&project, &Progress::start(&project));
        assert_eq!(
            briefing,
            Briefing {
                project: project.id().clone(),
                title: text("Bike registration"),
                current: Some(CurrentStep {
                    id: step_id("investigate"),
                    title: text("investigate title"),
                    guide: text("investigate guide"),
                    state: StepState::Open,
                }),
                next: Some(NextStep { id: step_id("plan"), title: text("plan title") }),
                handoff_note: None,
            }
        );
    }

    #[test]
    fn an_approved_step_stays_current_until_its_stamp() {
        let project = bike_registration();
        let mut progress = Progress::start(&project);
        progress.submit(&project, all_pass(&project, "investigate"), AtStamp).expect("approved");

        let briefing = Briefing::new(&project, &progress);
        let current = briefing.current.expect("a current step");
        assert_eq!((current.id, current.state), (step_id("investigate"), StepState::Approved));
        assert_eq!(
            briefing.handoff_note.map(|note| note.as_str().to_owned()),
            Some("investigate is done".to_owned())
        );
    }

    #[test]
    fn the_next_step_carries_the_previous_handoff_note() {
        let project = bike_registration();
        let mut progress = Progress::start(&project);
        progress.submit(&project, all_pass(&project, "investigate"), AtApproval).expect("done");

        let briefing = Briefing::new(&project, &progress);
        assert_eq!(briefing.current.map(|step| step.id), Some(step_id("plan")));
        assert_eq!(briefing.next, None);
        assert!(briefing.handoff_note.is_some());
    }

    #[test]
    fn a_finished_project_has_no_current_step() {
        let project = bike_registration();
        let mut progress = Progress::start(&project);
        progress.submit(&project, all_pass(&project, "investigate"), AtApproval).expect("done");
        progress.submit(&project, all_pass(&project, "plan"), AtApproval).expect("done");

        let briefing = Briefing::new(&project, &progress);
        assert_eq!((briefing.current, briefing.next), (None, None));
    }
}
