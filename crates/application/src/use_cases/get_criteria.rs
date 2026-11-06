//! The open step's criteria, for the agent to check its finished work against (M7). Given only
//! on request, and without hints (ADR 0006 #5, #7).

use domain::checklist::{Checklist, ChecklistError};
use domain::player::PlayerId;
use domain::project::{ProjectId, StepId};

use crate::ports::PortError;
use crate::ports::catalog::Catalog;
use crate::ports::progress::ProgressStore;

#[derive(Debug, Clone)]
pub struct Input {
    pub player: PlayerId,
    pub project: ProjectId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output {
    pub checklist: Checklist,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    UnknownProject(ProjectId),
    /// The player hasn't started this project.
    NotStarted(ProjectId),
    /// No step takes a report. `current` is the approved step waiting for its stamp, or `None`
    /// once the project is finished.
    NoOpenStep {
        current: Option<StepId>,
    },
    Port(PortError),
}

pub async fn run(
    catalog: &dyn Catalog,
    progress_store: &dyn ProgressStore,
    input: Input,
) -> Result<Output, Error> {
    let project = catalog
        .project(&input.project)
        .await
        .map_err(Error::Port)?
        .ok_or_else(|| Error::UnknownProject(input.project.clone()))?;
    let (progress, _) = progress_store
        .load(&input.player, &input.project)
        .await
        .map_err(Error::Port)?
        .ok_or_else(|| Error::NotStarted(input.project.clone()))?;
    let checklist = Checklist::new(&project, &progress).map_err(|error| match error {
        ChecklistError::NoOpenStep { current } => Error::NoOpenStep { current },
    })?;
    Ok(Output { checklist })
}

#[cfg(test)]
mod tests {
    use domain::stamp::StepCompletion;

    use super::*;
    use crate::testkit::catalog::FakeCatalog;
    use crate::testkit::fixtures;
    use crate::testkit::progress::FakeProgress;

    fn input() -> Input {
        Input { player: fixtures::player(), project: fixtures::project().id().clone() }
    }

    #[tokio::test]
    async fn the_open_step_criteria_are_given() {
        let project = fixtures::project();
        let catalog = FakeCatalog::new(vec![project.clone()]);
        let store = FakeProgress::new();
        store.save(&fixtures::player(), &fixtures::started(), None).await.expect("first save");

        let checklist = run(&catalog, &store, input()).await.expect("a checklist").checklist;
        let step = project.step(&fixtures::first_step()).expect("the first step");
        assert_eq!(checklist.step, step.id);
        let criteria: Vec<_> = checklist.items.iter().map(|item| &item.criterion).collect();
        assert_eq!(criteria, step.criteria.iter().map(|criterion| &criterion.id).collect::<Vec<_>>());
    }

    #[tokio::test]
    async fn an_approved_step_has_no_criteria_to_check() {
        let project = fixtures::project();
        let catalog = FakeCatalog::new(vec![project.clone()]);
        let mut progress = fixtures::started();
        progress
            .submit(&project, fixtures::all_pass(&fixtures::first_step()), StepCompletion::AtStamp)
            .expect("approved");
        let store = FakeProgress::new();
        store.save(&fixtures::player(), &progress, None).await.expect("first save");

        let output = run(&catalog, &store, input()).await;
        assert_eq!(output, Err(Error::NoOpenStep { current: Some(fixtures::first_step()) }));
    }

    #[tokio::test]
    async fn an_unknown_project_is_refused() {
        let catalog = FakeCatalog::new(Vec::new());
        let output = run(&catalog, &FakeProgress::new(), input()).await;
        assert_eq!(output, Err(Error::UnknownProject(fixtures::project().id().clone())));
    }

    #[tokio::test]
    async fn a_project_not_started_is_refused() {
        let catalog = FakeCatalog::new(vec![fixtures::project()]);
        let output = run(&catalog, &FakeProgress::new(), input()).await;
        assert_eq!(output, Err(Error::NotStarted(fixtures::project().id().clone())));
    }
}
