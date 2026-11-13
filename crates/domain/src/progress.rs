//! A player's progress in one project, and the rules that move it.

use std::collections::HashSet;

use crate::project::{CriterionId, Project, ProjectId, StepId};
use crate::report::{HandoffNote, StepReport, SubmitError, SubmitReport, Submitted, Verdict};
use crate::stamp::{StampError, StepCompletion};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepState {
    Locked,
    Open,
    /// Every criterion passed; waiting for the stamp.
    Approved,
    Done,
    Skipped,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Progress {
    project: ProjectId,
    /// One entry per project step, in the project's order.
    steps: Vec<StepProgress>,
    last_handoff_note: Option<HandoffNote>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StepProgress {
    step: StepId,
    state: StepState,
    latest_report: Option<StepReport>,
}

impl Progress {
    /// The first step is open and the rest are locked.
    pub fn start(project: &Project) -> Self {
        let steps = project
            .steps()
            .iter()
            .enumerate()
            .map(|(index, step)| StepProgress {
                step: step.id.clone(),
                state: if index == 0 { StepState::Open } else { StepState::Locked },
                latest_report: None,
            })
            .collect();
        Self { project: project.id().clone(), steps, last_handoff_note: None }
    }

    pub fn project(&self) -> &ProjectId {
        &self.project
    }

    pub fn state(&self, step: &StepId) -> Option<StepState> {
        self.find(step).map(|index| self.steps[index].state)
    }

    pub fn latest_report(&self, step: &StepId) -> Option<&StepReport> {
        self.find(step).and_then(|index| self.steps[index].latest_report.as_ref())
    }

    /// The step being worked on: the open one, or the approved one waiting for its stamp.
    pub fn current_step(&self) -> Option<&StepId> {
        self.current_index().map(|index| &self.steps[index].step)
    }

    /// The step after the current one, for the briefing's preview.
    pub fn next_step(&self) -> Option<&StepId> {
        self.current_index().and_then(|index| self.steps.get(index + 1)).map(|entry| &entry.step)
    }

    /// The note from the last approved step, for the next briefing.
    pub fn last_handoff_note(&self) -> Option<&HandoffNote> {
        self.last_handoff_note.as_ref()
    }

    /// Records a report on the open step. A fail is recorded and leaves the step open; an all-pass
    /// report with a handoff note approves it, and when steps complete at approval also completes
    /// it and opens the next step. A report that repeats its step's latest one gets the answer that
    /// one got (E4). A repeat or a refused report changes nothing. `completion` comes from
    /// `settings::step_completion`.
    pub fn submit(
        &mut self,
        project: &Project,
        report: SubmitReport,
        completion: StepCompletion,
    ) -> Result<Submitted, SubmitError> {
        if let Some(answer) = self.earlier_answer(&report) {
            return Ok(answer);
        }
        let index = self.find(&report.step).ok_or_else(|| SubmitError::UnknownStep(report.step.clone()))?;
        if self.steps[index].state != StepState::Open {
            return Err(SubmitError::StepNotOpen { open: self.open_step().cloned() });
        }
        let expected = criteria_of(project, &report.step);
        if !covers_exactly(&report.verdicts, &expected) {
            return Err(SubmitError::CriteriaMismatch { expected });
        }
        let failed = not_passed(&expected, &report.verdicts);
        if failed.is_empty() && report.handoff_note.is_none() {
            return Err(SubmitError::HandoffNoteMissing);
        }

        let answer = if failed.is_empty() {
            self.last_handoff_note = report.handoff_note.clone();
            match completion {
                StepCompletion::AtStamp => {
                    self.steps[index].state = StepState::Approved;
                    Submitted::StepApproved
                }
                StepCompletion::AtApproval => {
                    self.steps[index].state = StepState::Done;
                    Submitted::StepDone { next: self.open_after(index) }
                }
            }
        } else {
            Submitted::StepStillOpen { failed }
        };
        self.steps[index].latest_report = Some(StepReport::accept(report, answer.clone()));
        Ok(answer)
    }

    /// The answer `submit` already gave `report`, when it repeats its step's latest report and that
    /// answer still holds (E4). Once the progress has moved on, there's none, and `submit` checks
    /// the repeat like any other report.
    pub fn earlier_answer(&self, report: &SubmitReport) -> Option<Submitted> {
        let index = self.find(&report.step)?;
        let entry = &self.steps[index];
        let latest = entry.latest_report.as_ref()?;
        let holds = match latest.answer() {
            Submitted::StepStillOpen { .. } => entry.state == StepState::Open,
            Submitted::StepApproved => entry.state == StepState::Approved,
            Submitted::StepDone { next } => {
                entry.state == StepState::Done && self.open_step() == next.as_ref()
            }
        };
        (holds && latest.is_repeated_by(report)).then(|| latest.answer().clone())
    }

    /// Whether a stamp may be issued for `step` now. `completion` comes from
    /// `settings::step_completion`.
    pub fn stamp_allowed(
        &self,
        project: &Project,
        step: &StepId,
        completion: StepCompletion,
    ) -> Result<(), StampError> {
        let index = self.find(step).ok_or_else(|| StampError::UnknownStep(step.clone()))?;
        if completion == StepCompletion::AtApproval {
            return Err(StampError::StampOff);
        }
        let entry = &self.steps[index];
        if entry.state == StepState::Approved {
            return Ok(());
        }
        let verdicts = entry.latest_report.as_ref().map_or(&[][..], StepReport::verdicts);
        let pending = not_passed(&criteria_of(project, step), verdicts);
        Err(StampError::NotApproved { pending })
    }

    fn find(&self, step: &StepId) -> Option<usize> {
        self.steps.iter().position(|entry| &entry.step == step)
    }

    fn current_index(&self) -> Option<usize> {
        self.steps.iter().position(|entry| matches!(entry.state, StepState::Open | StepState::Approved))
    }

    /// The step that takes a report, if one is open.
    pub fn open_step(&self) -> Option<&StepId> {
        self.steps.iter().find(|entry| entry.state == StepState::Open).map(|entry| &entry.step)
    }

    /// Opens the first locked step after `index`, and returns it.
    fn open_after(&mut self, index: usize) -> Option<StepId> {
        let next = self.steps.iter_mut().skip(index + 1).find(|entry| entry.state == StepState::Locked)?;
        next.state = StepState::Open;
        Some(next.step.clone())
    }
}

fn criteria_of(project: &Project, step: &StepId) -> Vec<CriterionId> {
    project
        .step(step)
        .map(|step| step.criteria.iter().map(|criterion| criterion.id.clone()).collect())
        .unwrap_or_default()
}

fn covers_exactly(verdicts: &[(CriterionId, Verdict)], expected: &[CriterionId]) -> bool {
    let reported: HashSet<&CriterionId> = verdicts.iter().map(|(id, _)| id).collect();
    reported.len() == verdicts.len()
        && reported.len() == expected.len()
        && expected.iter().all(|id| reported.contains(id))
}

/// The criteria in `expected` that `verdicts` don't pass, in `expected`'s order.
fn not_passed(expected: &[CriterionId], verdicts: &[(CriterionId, Verdict)]) -> Vec<CriterionId> {
    let passed = |id: &CriterionId| verdicts.iter().any(|(r, v)| r == id && *v == Verdict::Pass);
    expected.iter().filter(|id| !passed(id)).cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::Verdict::{Fail, Pass};
    use crate::stamp::StepCompletion::{AtApproval, AtStamp};
    use crate::testkit::builders::{criterion_id, report, step_id};
    use crate::testkit::fixtures::bike_registration;

    fn all_pass(note: Option<&str>) -> SubmitReport {
        report("investigate", &[("reads-rules", Pass), ("lists-fields", Pass)], note)
    }

    #[test]
    fn starting_opens_the_first_step_only() {
        let progress = Progress::start(&bike_registration());
        assert_eq!(progress.state(&step_id("investigate")), Some(StepState::Open));
        assert_eq!(progress.state(&step_id("plan")), Some(StepState::Locked));
        assert_eq!(progress.current_step(), Some(&step_id("investigate")));
        assert_eq!(progress.next_step(), Some(&step_id("plan")));
        assert_eq!(progress.last_handoff_note(), None);
    }

    #[test]
    fn a_fail_is_recorded_and_the_step_stays_open() {
        let project = bike_registration();
        let mut progress = Progress::start(&project);
        let failing = report("investigate", &[("reads-rules", Pass), ("lists-fields", Fail)], None);

        let outcome = progress.submit(&project, failing.clone(), AtApproval);

        assert_eq!(outcome, Ok(Submitted::StepStillOpen { failed: vec![criterion_id("lists-fields")] }));
        assert_eq!(progress.state(&step_id("investigate")), Some(StepState::Open));
        assert_eq!(
            progress.latest_report(&step_id("investigate")).map(StepReport::verdicts),
            Some(&failing.verdicts[..])
        );
    }

    #[test]
    fn all_pass_with_the_stamp_off_completes_the_step_and_opens_the_next() {
        let project = bike_registration();
        let mut progress = Progress::start(&project);

        let outcome = progress.submit(&project, all_pass(Some("Fields listed in FIELDS.md")), AtApproval);

        assert_eq!(outcome, Ok(Submitted::StepDone { next: Some(step_id("plan")) }));
        assert_eq!(progress.state(&step_id("investigate")), Some(StepState::Done));
        assert_eq!(progress.state(&step_id("plan")), Some(StepState::Open));
        assert_eq!(progress.current_step(), Some(&step_id("plan")));
        assert_eq!(progress.next_step(), None);
        assert_eq!(progress.last_handoff_note().map(HandoffNote::as_str), Some("Fields listed in FIELDS.md"));
    }

    #[test]
    fn all_pass_with_the_stamp_on_stops_at_approved() {
        let project = bike_registration();
        let mut progress = Progress::start(&project);

        let outcome = progress.submit(&project, all_pass(Some("Done")), AtStamp);

        assert_eq!(outcome, Ok(Submitted::StepApproved));
        assert_eq!(progress.state(&step_id("investigate")), Some(StepState::Approved));
        assert_eq!(progress.state(&step_id("plan")), Some(StepState::Locked));
        assert_eq!(progress.current_step(), Some(&step_id("investigate")));
    }

    #[test]
    fn the_last_step_done_leaves_nothing_open() {
        let project = bike_registration();
        let mut progress = Progress::start(&project);
        progress.submit(&project, all_pass(Some("Done")), AtApproval).expect("first step done");

        let last = report("plan", &[("has-plan", Pass), ("plan-reviewed", Pass)], Some("Plan in PLAN.md"));
        let outcome = progress.submit(&project, last, AtApproval);

        assert_eq!(outcome, Ok(Submitted::StepDone { next: None }));
        assert_eq!(progress.current_step(), None);
    }

    #[test]
    fn all_pass_without_a_note_is_refused_and_not_recorded() {
        let project = bike_registration();
        let mut progress = Progress::start(&project);

        assert_eq!(
            progress.submit(&project, all_pass(None), AtApproval),
            Err(SubmitError::HandoffNoteMissing)
        );
        assert_eq!(
            progress.submit(&project, all_pass(Some("   ")), AtApproval),
            Err(SubmitError::HandoffNoteMissing)
        );
        assert_eq!(progress, Progress::start(&project));
    }

    #[test]
    fn a_report_on_a_step_that_isnt_open_names_the_open_step() {
        let project = bike_registration();
        let mut progress = Progress::start(&project);
        progress.submit(&project, all_pass(Some("Done")), AtApproval).expect("first step done");

        let outcome = progress.submit(&project, all_pass(Some("Again")), AtApproval);

        assert_eq!(outcome, Err(SubmitError::StepNotOpen { open: Some(step_id("plan")) }));
    }

    #[test]
    fn a_repeated_fail_gets_the_same_answer_and_changes_nothing() {
        let project = bike_registration();
        let mut progress = Progress::start(&project);
        let failing = report("investigate", &[("reads-rules", Pass), ("lists-fields", Fail)], None);
        progress.submit(&project, failing.clone(), AtApproval).expect("recorded");
        let recorded = progress.clone();
        let answer = Submitted::StepStillOpen { failed: vec![criterion_id("lists-fields")] };

        assert_eq!(progress.earlier_answer(&failing), Some(answer.clone()));
        assert_eq!(progress.submit(&project, failing, AtApproval), Ok(answer));
        assert_eq!(progress, recorded);
    }

    #[test]
    fn a_repeated_all_pass_gets_the_same_answer_and_changes_nothing() {
        let project = bike_registration();
        let answers = [
            (AtStamp, Submitted::StepApproved),
            (AtApproval, Submitted::StepDone { next: Some(step_id("plan")) }),
        ];
        for (completion, answer) in answers {
            let mut progress = Progress::start(&project);
            progress.submit(&project, all_pass(Some("Done")), completion).expect("recorded");
            let recorded = progress.clone();

            let outcome = progress.submit(&project, all_pass(Some("Done")), completion);

            assert_eq!(outcome, Ok(answer));
            assert_eq!(progress, recorded);
        }
    }

    #[test]
    fn verdicts_in_another_order_are_the_same_report() {
        let project = bike_registration();
        let mut progress = Progress::start(&project);
        progress.submit(&project, all_pass(Some("Done")), AtStamp).expect("approved");
        let reordered = report("investigate", &[("lists-fields", Pass), ("reads-rules", Pass)], Some("Done"));

        assert_eq!(progress.earlier_answer(&reordered), Some(Submitted::StepApproved));
    }

    #[test]
    fn a_report_that_differs_from_the_latest_is_not_a_repeat() {
        let project = bike_registration();
        let mut progress = Progress::start(&project);
        progress.submit(&project, all_pass(Some("Done")), AtStamp).expect("approved");

        let other_note = all_pass(Some("Again"));
        let other_verdict =
            report("investigate", &[("reads-rules", Pass), ("lists-fields", Fail)], Some("Done"));
        let duplicated = report(
            "investigate",
            &[("reads-rules", Pass), ("lists-fields", Pass), ("reads-rules", Pass)],
            Some("Done"),
        );
        let extra = report(
            "investigate",
            &[("reads-rules", Pass), ("lists-fields", Pass), ("has-plan", Pass)],
            Some("Done"),
        );

        for other in [other_note, other_verdict, duplicated, extra] {
            assert_eq!(progress.earlier_answer(&other), None);
        }
    }

    #[test]
    fn a_repeat_after_the_project_moved_on_is_checked_like_any_report() {
        let project = bike_registration();
        let mut progress = Progress::start(&project);
        progress.submit(&project, all_pass(Some("Done")), AtApproval).expect("first step done");
        let last = report("plan", &[("has-plan", Pass), ("plan-reviewed", Pass)], Some("Plan in PLAN.md"));
        progress.submit(&project, last, AtApproval).expect("last step done");

        assert_eq!(progress.earlier_answer(&all_pass(Some("Done"))), None);
        assert_eq!(
            progress.submit(&project, all_pass(Some("Done")), AtApproval),
            Err(SubmitError::StepNotOpen { open: None })
        );
    }

    #[test]
    fn a_report_on_an_unknown_step_is_refused() {
        let project = bike_registration();
        let mut progress = Progress::start(&project);

        let outcome = progress.submit(&project, report("deploy", &[("reads-rules", Pass)], None), AtApproval);

        assert_eq!(outcome, Err(SubmitError::UnknownStep(step_id("deploy"))));
        assert_eq!(progress, Progress::start(&project));
    }

    #[test]
    fn verdicts_must_cover_exactly_the_steps_criteria() {
        let project = bike_registration();
        let mut progress = Progress::start(&project);
        let expected = Err(SubmitError::CriteriaMismatch {
            expected: vec![criterion_id("reads-rules"), criterion_id("lists-fields")],
        });

        let missing = report("investigate", &[("reads-rules", Pass)], Some("note"));
        let foreign = report("investigate", &[("reads-rules", Pass), ("has-plan", Pass)], Some("note"));
        let twice = report("investigate", &[("reads-rules", Pass), ("reads-rules", Pass)], Some("note"));
        let extra = report(
            "investigate",
            &[("reads-rules", Pass), ("lists-fields", Pass), ("has-plan", Pass)],
            Some("note"),
        );

        for wrong in [missing, foreign, twice, extra] {
            assert_eq!(progress.submit(&project, wrong, AtApproval), expected);
        }
        assert_eq!(progress, Progress::start(&project));
    }

    #[test]
    fn a_stamp_is_allowed_once_the_step_is_approved() {
        let project = bike_registration();
        let mut progress = Progress::start(&project);
        progress.submit(&project, all_pass(Some("Done")), AtStamp).expect("approved");

        assert_eq!(progress.stamp_allowed(&project, &step_id("investigate"), AtStamp), Ok(()));
    }

    #[test]
    fn a_stamp_before_approval_names_the_criteria_not_passed() {
        let project = bike_registration();
        let mut progress = Progress::start(&project);
        let not_yet = Err(StampError::NotApproved {
            pending: vec![criterion_id("reads-rules"), criterion_id("lists-fields")],
        });
        assert_eq!(progress.stamp_allowed(&project, &step_id("investigate"), AtStamp), not_yet);

        let failing = report("investigate", &[("reads-rules", Pass), ("lists-fields", Fail)], None);
        progress.submit(&project, failing, AtStamp).expect("recorded");

        assert_eq!(
            progress.stamp_allowed(&project, &step_id("investigate"), AtStamp),
            Err(StampError::NotApproved { pending: vec![criterion_id("lists-fields")] })
        );
    }

    #[test]
    fn a_stamp_is_refused_when_the_stamp_is_off() {
        let project = bike_registration();
        let mut progress = Progress::start(&project);
        progress.submit(&project, all_pass(Some("Done")), AtApproval).expect("done");

        assert_eq!(
            progress.stamp_allowed(&project, &step_id("investigate"), AtApproval),
            Err(StampError::StampOff)
        );
    }

    #[test]
    fn a_stamp_for_an_unknown_step_is_refused() {
        let project = bike_registration();
        let progress = Progress::start(&project);

        assert_eq!(
            progress.stamp_allowed(&project, &step_id("deploy"), AtStamp),
            Err(StampError::UnknownStep(step_id("deploy")))
        );
    }
}
