//! Progress held in memory, with a version per save.

use std::collections::HashMap;
use std::sync::Mutex;

use application::ports::PortError;
use application::ports::progress::{ProgressStore, SaveProgressError, Version};
use async_trait::async_trait;
use domain::player::PlayerId;
use domain::progress::Progress;
use domain::project::ProjectId;

use crate::lock;

#[derive(Default)]
pub struct MemoryProgress {
    saved: Mutex<HashMap<(PlayerId, ProjectId), (Progress, Version)>>,
}

impl MemoryProgress {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl ProgressStore for MemoryProgress {
    async fn started(&self, player: &PlayerId) -> Result<Vec<ProjectId>, PortError> {
        let saved = lock(&self.saved)?;
        Ok(saved.keys().filter(|(owner, _)| owner == player).map(|(_, project)| project.clone()).collect())
    }

    async fn load(
        &self,
        player: &PlayerId,
        project: &ProjectId,
    ) -> Result<Option<(Progress, Version)>, PortError> {
        let saved = lock(&self.saved)?;
        Ok(saved.get(&(player.clone(), project.clone())).cloned())
    }

    async fn save(
        &self,
        player: &PlayerId,
        progress: &Progress,
        loaded: Option<Version>,
    ) -> Result<Version, SaveProgressError> {
        // One lock covers the check and the write, so two saves can't both pass the check.
        let mut saved = lock(&self.saved).map_err(SaveProgressError::Port)?;
        let key = (player.clone(), progress.project().clone());
        if saved.get(&key).map(|(_, version)| *version) != loaded {
            return Err(SaveProgressError::Conflict);
        }
        let version = loaded.map_or(Version::FIRST, Version::next);
        saved.insert(key, (progress.clone(), version));
        Ok(version)
    }
}

#[cfg(test)]
mod tests {
    use application::testkit::progress::suite;

    use super::MemoryProgress;

    #[tokio::test]
    async fn passes_the_progress_suite() {
        suite(MemoryProgress::new).await;
    }
}
