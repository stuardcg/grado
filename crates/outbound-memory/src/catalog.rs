//! Projects held in memory.

use application::ports::PortError;
use application::ports::catalog::Catalog;
use async_trait::async_trait;
use domain::project::{Project, ProjectId};

pub struct MemoryCatalog {
    projects: Vec<Project>,
}

impl MemoryCatalog {
    pub fn new(projects: Vec<Project>) -> Self {
        Self { projects }
    }
}

#[async_trait]
impl Catalog for MemoryCatalog {
    async fn projects(&self) -> Result<Vec<Project>, PortError> {
        Ok(self.projects.clone())
    }

    async fn project(&self, id: &ProjectId) -> Result<Option<Project>, PortError> {
        Ok(self.projects.iter().find(|project| project.id() == id).cloned())
    }
}

#[cfg(test)]
mod tests {
    use application::testkit::catalog::suite;

    use super::MemoryCatalog;

    #[tokio::test]
    async fn passes_the_catalog_suite() {
        suite(MemoryCatalog::new).await;
    }
}
