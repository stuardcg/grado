//! The progress suite and its fake.

use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use domain::player::PlayerId;
use domain::progress::Progress;
use domain::project::ProjectId;
use domain::report::{SubmitReport, Verdict};
use domain::stamp::StepCompletion;

use super::fixtures;
use crate::ports::PortError;
use crate::ports::progress::{ProgressStore, SaveProgressError, Version};

/// Checks an empty `ProgressStore` built by `make`.
pub async fn suite<P: ProgressStore>(make: impl Fn() -> P) {
    let player = fixtures::player();
    let project = fixtures::project();
    let started = fixtures::started();

    let store = make();
    assert!(store.load(&player, project.id()).await.expect("load").is_none(), "not started");
    assert!(store.started(&player).await.expect("started").is_empty(), "nothing started");

    let first = store.save(&player, &started, None).await.expect("first save");
    assert_eq!(first, Version::FIRST);
    assert_eq!(store.load(&player, project.id()).await.expect("load"), Some((started.clone(), first)));
    assert_eq!(store.started(&player).await.expect("started"), vec![project.id().clone()]);

    let reported = failed_report(&started);
    let second = store.save(&player, &reported, Some(first)).await.expect("save on the loaded version");
    assert_eq!(second, first.next());
    assert_eq!(store.load(&player, project.id()).await.expect("load"), Some((reported.clone(), second)));

    // Another connection loaded `first` too, and saves after `second` (E5, ADR 0004 rule 25).
    let stale = store.save(&player, &started, Some(first)).await;
    assert_eq!(stale, Err(SaveProgressError::Conflict), "a save on a stale version");
    assert_eq!(store.load(&player, project.id()).await.expect("load"), Some((reported, second)));

    let again = store.save(&player, &started, None).await;
    assert_eq!(again, Err(SaveProgressError::Conflict), "creating a progress that exists");

    let other = PlayerId::new("grace").expect("not blank");
    let missing = store.save(&other, &started, Some(first)).await;
    assert_eq!(missing, Err(SaveProgressError::Conflict), "saving a progress never created");
    assert!(store.load(&other, project.id()).await.expect("load").is_none(), "players are apart");
    assert!(store.started(&other).await.expect("started").is_empty(), "players are apart");
}

/// `progress` with a failed report on its open step, so it differs from what was saved before.
fn failed_report(progress: &Progress) -> Progress {
    let project = fixtures::project();
    let step = &project.steps()[0];
    let verdicts = step.criteria.iter().map(|criterion| (criterion.id.clone(), Verdict::Fail)).collect();
    let mut progress = progress.clone();
    progress
        .submit(
            &project,
            SubmitReport { step: step.id.clone(), verdicts, handoff_note: None },
            StepCompletion::AtStamp,
        )
        .expect("a failed report on the open step");
    progress
}

/// Holds progress in memory, refusing stale saves like any store.
#[derive(Default)]
pub struct FakeProgress {
    saved: Mutex<HashMap<(PlayerId, ProjectId), (Progress, Version)>>,
}

impl FakeProgress {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl ProgressStore for FakeProgress {
    async fn started(&self, player: &PlayerId) -> Result<Vec<ProjectId>, PortError> {
        let saved = self.saved.lock().expect("no test panics while holding the lock");
        Ok(saved.keys().filter(|(owner, _)| owner == player).map(|(_, project)| project.clone()).collect())
    }

    async fn load(
        &self,
        player: &PlayerId,
        project: &ProjectId,
    ) -> Result<Option<(Progress, Version)>, PortError> {
        let saved = self.saved.lock().expect("no test panics while holding the lock");
        Ok(saved.get(&(player.clone(), project.clone())).cloned())
    }

    async fn save(
        &self,
        player: &PlayerId,
        progress: &Progress,
        loaded: Option<Version>,
    ) -> Result<Version, SaveProgressError> {
        let mut saved = self.saved.lock().expect("no test panics while holding the lock");
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
    use super::*;

    #[tokio::test]
    async fn the_fake_passes_the_suite() {
        suite(FakeProgress::new).await;
    }
}
