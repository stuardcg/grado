//! The predefined projects.

use async_trait::async_trait;
use domain::project::{Project, ProjectId};

use super::PortError;

#[async_trait]
pub trait Catalog: Send + Sync {
    /// Every project, in catalog order.
    async fn projects(&self) -> Result<Vec<Project>, PortError>;

    async fn project(&self, id: &ProjectId) -> Result<Option<Project>, PortError>;
}
