//! What a fresh agent needs to start or resume work on a project: the current step, a preview of
//! the next one and the last handoff note, but no criteria (M5, E7, ADR 0006 #5).

use domain::briefing::Briefing;
use domain::player::PlayerId;
use domain::project::ProjectId;

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
    pub briefing: Briefing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    UnknownProject(ProjectId),
    /// The player hasn't started this project.
    NotStarted(ProjectId),
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
    Ok(Output { briefing: Briefing::new(&project, &progress) })
}

#[cfg(test)]
mod tests {
    use domain::progress::StepState;

    use super::*;
    use crate::testkit::catalog::FakeCatalog;
    use crate::testkit::fixtures;
    use crate::testkit::progress::FakeProgress;

    fn input() -> Input {
        Input { player: fixtures::player(), project: fixtures::project().id().clone() }
    }

    #[tokio::test]
    async fn a_started_player_is_briefed_on_the_open_step() {
        let catalog = FakeCatalog::new(vec![fixtures::project()]);
        let store = FakeProgress::new();
        store.save(&fixtures::player(), &fixtures::started(), None).await.expect("first save");

        let briefing = run(&catalog, &store, input()).await.expect("a briefing").briefing;
        let current = briefing.current.expect("a current step");
        assert_eq!((current.id, current.state), (fixtures::first_step(), StepState::Open));
        assert_eq!(briefing.next.map(|step| step.id), Some(fixtures::second_step()));
        assert_eq!(briefing.handoff_note, None);
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
