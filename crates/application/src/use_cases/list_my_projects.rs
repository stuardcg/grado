//! The projects a player has started, so the agent can find the one it's working on (M5).

use domain::player::PlayerId;
use domain::project::ProjectId;
use shared::text::NonBlankString;

use crate::ports::PortError;
use crate::ports::catalog::Catalog;
use crate::ports::progress::ProgressStore;

#[derive(Debug, Clone)]
pub struct Input {
    pub player: PlayerId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output {
    /// Each started project's id and title, in no particular order.
    pub projects: Vec<(ProjectId, NonBlankString)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The player started a project the catalog no longer has.
    UnknownProject(ProjectId),
    Port(PortError),
}

pub async fn run(
    catalog: &dyn Catalog,
    progress_store: &dyn ProgressStore,
    input: Input,
) -> Result<Output, Error> {
    let started = progress_store.started(&input.player).await.map_err(Error::Port)?;
    let mut projects = Vec::new();
    for id in started {
        let project = catalog
            .project(&id)
            .await
            .map_err(Error::Port)?
            .ok_or_else(|| Error::UnknownProject(id.clone()))?;
        projects.push((id, project.title().clone()));
    }
    Ok(Output { projects })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::catalog::FakeCatalog;
    use crate::testkit::fixtures;
    use crate::testkit::progress::FakeProgress;

    async fn started() -> FakeProgress {
        let store = FakeProgress::new();
        store.save(&fixtures::player(), &fixtures::started(), None).await.expect("first save");
        store
    }

    #[tokio::test]
    async fn a_started_project_is_listed_with_its_title() {
        let project = fixtures::project();
        let catalog = FakeCatalog::new(vec![project.clone()]);
        let output = run(&catalog, &started().await, Input { player: fixtures::player() }).await;
        assert_eq!(output, Ok(Output { projects: vec![(project.id().clone(), project.title().clone())] }));
    }

    #[tokio::test]
    async fn a_player_who_started_nothing_has_no_projects() {
        let catalog = FakeCatalog::new(vec![fixtures::project()]);
        let output = run(&catalog, &FakeProgress::new(), Input { player: fixtures::player() }).await;
        assert_eq!(output, Ok(Output { projects: Vec::new() }));
    }

    #[tokio::test]
    async fn a_started_project_missing_from_the_catalog_is_refused() {
        let catalog = FakeCatalog::new(Vec::new());
        let output = run(&catalog, &started().await, Input { player: fixtures::player() }).await;
        assert_eq!(output, Err(Error::UnknownProject(fixtures::project().id().clone())));
    }
}
