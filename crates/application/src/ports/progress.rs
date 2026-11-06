//! Each player's progress in each project they started.

use async_trait::async_trait;
use domain::player::PlayerId;
use domain::progress::Progress;
use domain::project::ProjectId;

use super::PortError;

/// Which save of a progress was loaded. A save names the version it builds on, so one that lost a
/// race with another connection is refused (ADR 0004 rule 25).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Version(u64);

impl Version {
    /// The version of a progress's first save.
    pub const FIRST: Self = Self(0);

    /// The version the save after this one gets.
    pub fn next(self) -> Self {
        Self(self.0 + 1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveProgressError {
    /// The progress was saved, or first created, since `loaded`.
    Conflict,
    Port(PortError),
}

#[async_trait]
pub trait ProgressStore: Send + Sync {
    /// The projects `player` has started, in no particular order.
    async fn started(&self, player: &PlayerId) -> Result<Vec<ProjectId>, PortError>;

    async fn load(
        &self,
        player: &PlayerId,
        project: &ProjectId,
    ) -> Result<Option<(Progress, Version)>, PortError>;

    /// Saves `progress` for its project. `loaded` is the version it was loaded at, or `None` for
    /// a progress that doesn't exist yet. Returns the new version.
    async fn save(
        &self,
        player: &PlayerId,
        progress: &Progress,
        loaded: Option<Version>,
    ) -> Result<Version, SaveProgressError>;
}
