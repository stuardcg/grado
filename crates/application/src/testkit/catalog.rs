//! The catalog suite and its fake.

use async_trait::async_trait;
use domain::project::{Project, ProjectId};

use super::fixtures;
use crate::ports::PortError;
use crate::ports::catalog::Catalog;

/// Checks a `Catalog` built by `make` from the projects it should hold.
pub async fn suite<C: Catalog>(make: impl Fn(Vec<Project>) -> C) {
    let first = fixtures::project();
    let second = renamed(&first, "habit-tracker-2");

    let catalog = make(vec![first.clone(), second.clone()]);
    let listed = catalog.projects().await.expect("projects");
    assert_eq!(ids(&listed), vec![first.id().clone(), second.id().clone()], "catalog order");

    let found = catalog.project(first.id()).await.expect("project").expect("a known project");
    assert_eq!(found.id(), first.id());
    assert_eq!(step_ids(&found), step_ids(&first), "the project keeps its steps");

    let unknown = ProjectId::new("no-such-project").expect("not blank");
    assert!(catalog.project(&unknown).await.expect("project").is_none(), "unknown project");

    let empty = make(Vec::new());
    assert!(empty.projects().await.expect("projects").is_empty(), "empty catalog");
}

fn renamed(project: &Project, id: &str) -> Project {
    let id = ProjectId::new(id).expect("not blank");
    Project::new(id, project.title().clone(), project.steps().to_vec()).expect("same steps, so still valid")
}

fn ids(projects: &[Project]) -> Vec<ProjectId> {
    projects.iter().map(|project| project.id().clone()).collect()
}

fn step_ids(project: &Project) -> Vec<String> {
    project.steps().iter().map(|step| step.id.as_str().to_owned()).collect()
}

/// Holds a fixed list of projects.
pub struct FakeCatalog {
    projects: Vec<Project>,
}

impl FakeCatalog {
    pub fn new(projects: Vec<Project>) -> Self {
        Self { projects }
    }
}

#[async_trait]
impl Catalog for FakeCatalog {
    async fn projects(&self) -> Result<Vec<Project>, PortError> {
        Ok(self.projects.clone())
    }

    async fn project(&self, id: &ProjectId) -> Result<Option<Project>, PortError> {
        Ok(self.projects.iter().find(|project| project.id() == id).cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn the_fake_passes_the_suite() {
        suite(FakeCatalog::new).await;
    }
}
