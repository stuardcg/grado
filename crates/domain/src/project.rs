//! Predefined projects: ordered steps, each with a guide and criteria.

use std::collections::HashSet;

use shared::id::Id;
use shared::text::NonBlankString;

pub type ProjectId = Id<Project>;
pub type StepId = Id<Step>;
pub type CriterionId = Id<Criterion>;

/// A predefined app to build. Always has at least one step, and every step and criterion id is
/// unique within it.
#[derive(Debug, Clone)]
pub struct Project {
    id: ProjectId,
    title: NonBlankString,
    steps: Vec<Step>,
}

#[derive(Debug, Clone)]
pub struct Step {
    pub id: StepId,
    pub title: NonBlankString,
    pub guide: NonBlankString,
    pub criteria: Vec<Criterion>,
}

/// One thing a step's work must satisfy. The hint is for the app only, never the agent.
#[derive(Debug, Clone)]
pub struct Criterion {
    pub id: CriterionId,
    pub title: NonBlankString,
    pub instructions: NonBlankString,
    pub hint: NonBlankString,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectError {
    NoSteps,
    StepWithoutCriteria(StepId),
    DuplicateStep(StepId),
    DuplicateCriterion(CriterionId),
}

impl Project {
    pub fn new(id: ProjectId, title: NonBlankString, steps: Vec<Step>) -> Result<Self, ProjectError> {
        if steps.is_empty() {
            return Err(ProjectError::NoSteps);
        }
        let mut step_ids = HashSet::new();
        let mut criterion_ids = HashSet::new();
        for step in &steps {
            if !step_ids.insert(&step.id) {
                return Err(ProjectError::DuplicateStep(step.id.clone()));
            }
            if step.criteria.is_empty() {
                return Err(ProjectError::StepWithoutCriteria(step.id.clone()));
            }
            for criterion in &step.criteria {
                if !criterion_ids.insert(&criterion.id) {
                    return Err(ProjectError::DuplicateCriterion(criterion.id.clone()));
                }
            }
        }
        Ok(Self { id, title, steps })
    }

    pub fn id(&self) -> &ProjectId {
        &self.id
    }

    pub fn title(&self) -> &str {
        self.title.as_str()
    }

    pub fn steps(&self) -> &[Step] {
        &self.steps
    }

    pub fn step(&self, id: &StepId) -> Option<&Step> {
        self.steps.iter().find(|step| &step.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(value: &str) -> NonBlankString {
        NonBlankString::new(value).expect("test text isn't blank")
    }

    fn step_id(value: &str) -> StepId {
        StepId::new(value).expect("test ids aren't blank")
    }

    fn criterion_id(value: &str) -> CriterionId {
        CriterionId::new(value).expect("test ids aren't blank")
    }

    fn criterion(id: &str) -> Criterion {
        Criterion {
            id: criterion_id(id),
            title: text(&format!("{id} title")),
            instructions: text(&format!("check {id}")),
            hint: text(&format!("{id} hint")),
        }
    }

    fn step(id: &str, criteria: &[&str]) -> Step {
        Step {
            id: step_id(id),
            title: text(&format!("{id} title")),
            guide: text(&format!("{id} guide")),
            criteria: criteria.iter().map(|c| criterion(c)).collect(),
        }
    }

    fn project(steps: Vec<Step>) -> Result<Project, ProjectError> {
        let id = ProjectId::new("bike-registration").expect("test ids aren't blank");
        Project::new(id, text("Bike registration"), steps)
    }

    #[test]
    fn a_valid_project_finds_its_steps() {
        let project = project(vec![step("investigate", &["reads-rules"]), step("plan", &["has-plan"])])
            .expect("valid project");
        assert_eq!(project.step(&step_id("plan")).map(|s| s.title.as_str()), Some("plan title"));
        assert!(project.step(&step_id("deploy")).is_none());
    }

    #[test]
    fn a_project_needs_steps() {
        assert_eq!(project(vec![]).unwrap_err(), ProjectError::NoSteps);
    }

    #[test]
    fn every_step_needs_criteria() {
        assert_eq!(
            project(vec![step("investigate", &[])]).unwrap_err(),
            ProjectError::StepWithoutCriteria(step_id("investigate"))
        );
    }

    #[test]
    fn step_ids_are_unique() {
        assert_eq!(
            project(vec![step("investigate", &["a"]), step("investigate", &["b"])]).unwrap_err(),
            ProjectError::DuplicateStep(step_id("investigate"))
        );
    }

    #[test]
    fn criterion_ids_are_unique_across_steps() {
        assert_eq!(
            project(vec![step("investigate", &["a"]), step("plan", &["a"])]).unwrap_err(),
            ProjectError::DuplicateCriterion(criterion_id("a"))
        );
    }
}
