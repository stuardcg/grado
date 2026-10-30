//! Predefined projects: ordered steps, each with a guide and criteria.

use std::collections::HashSet;

use shared::id::Id;

pub type ProjectId = Id<Project>;
pub type StepId = Id<Step>;
pub type CriterionId = Id<Criterion>;

/// A predefined app to build. Always has at least one step, and every step and criterion id is
/// unique within it.
#[derive(Debug, Clone)]
pub struct Project {
    id: ProjectId,
    title: String,
    steps: Vec<Step>,
}

#[derive(Debug, Clone)]
pub struct Step {
    pub id: StepId,
    pub title: String,
    pub guide: String,
    pub criteria: Vec<Criterion>,
}

/// One thing a step's work must satisfy. The hint is for the app only, never the agent.
#[derive(Debug, Clone)]
pub struct Criterion {
    pub id: CriterionId,
    pub title: String,
    pub instructions: String,
    pub hint: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectInvalid {
    NoSteps,
    StepWithoutCriteria(StepId),
    DuplicateStep(StepId),
    DuplicateCriterion(CriterionId),
}

impl Project {
    pub fn new(id: ProjectId, title: String, steps: Vec<Step>) -> Result<Self, ProjectInvalid> {
        if steps.is_empty() {
            return Err(ProjectInvalid::NoSteps);
        }
        let mut step_ids = HashSet::new();
        let mut criterion_ids = HashSet::new();
        for step in &steps {
            if !step_ids.insert(&step.id) {
                return Err(ProjectInvalid::DuplicateStep(step.id.clone()));
            }
            if step.criteria.is_empty() {
                return Err(ProjectInvalid::StepWithoutCriteria(step.id.clone()));
            }
            for criterion in &step.criteria {
                if !criterion_ids.insert(&criterion.id) {
                    return Err(ProjectInvalid::DuplicateCriterion(criterion.id.clone()));
                }
            }
        }
        Ok(Self { id, title, steps })
    }

    pub fn id(&self) -> &ProjectId {
        &self.id
    }

    pub fn title(&self) -> &str {
        &self.title
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

    fn criterion(id: &str) -> Criterion {
        Criterion {
            id: CriterionId::new(id),
            title: format!("{id} title"),
            instructions: format!("check {id}"),
            hint: format!("{id} hint"),
        }
    }

    fn step(id: &str, criteria: &[&str]) -> Step {
        Step {
            id: StepId::new(id),
            title: format!("{id} title"),
            guide: format!("{id} guide"),
            criteria: criteria.iter().map(|c| criterion(c)).collect(),
        }
    }

    fn project(steps: Vec<Step>) -> Result<Project, ProjectInvalid> {
        Project::new(ProjectId::new("bike-registration"), "Bike registration".into(), steps)
    }

    #[test]
    fn a_valid_project_finds_its_steps() {
        let project = project(vec![step("investigate", &["reads-rules"]), step("plan", &["has-plan"])])
            .expect("valid project");
        assert_eq!(project.step(&StepId::new("plan")).map(|s| s.title.as_str()), Some("plan title"));
        assert!(project.step(&StepId::new("deploy")).is_none());
    }

    #[test]
    fn a_project_needs_steps() {
        assert_eq!(project(vec![]).unwrap_err(), ProjectInvalid::NoSteps);
    }

    #[test]
    fn every_step_needs_criteria() {
        assert_eq!(
            project(vec![step("investigate", &[])]).unwrap_err(),
            ProjectInvalid::StepWithoutCriteria(StepId::new("investigate"))
        );
    }

    #[test]
    fn step_ids_are_unique() {
        assert_eq!(
            project(vec![step("investigate", &["a"]), step("investigate", &["b"])]).unwrap_err(),
            ProjectInvalid::DuplicateStep(StepId::new("investigate"))
        );
    }

    #[test]
    fn criterion_ids_are_unique_across_steps() {
        assert_eq!(
            project(vec![step("investigate", &["a"]), step("plan", &["a"])]).unwrap_err(),
            ProjectInvalid::DuplicateCriterion(CriterionId::new("a"))
        );
    }
}
