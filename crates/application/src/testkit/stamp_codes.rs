//! The stamp codes suite and its fake.

use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use domain::player::PlayerId;
use domain::project::{ProjectId, StepId};
use domain::stamp::StampCode;

use super::fixtures;
use crate::ports::PortError;
use crate::ports::stamp_codes::StampCodes;

/// Checks an empty `StampCodes` built by `make`.
pub async fn suite<S: StampCodes>(make: impl Fn() -> S) {
    let player = fixtures::player();
    let project = fixtures::project().id().clone();
    let step = fixtures::first_step();
    let codes = make();
    assert_eq!(codes.current(&player, &project, &step).await.expect("current"), None, "none issued");

    let first = codes.issue(&player, &project, &step).await.expect("issue");
    let again = codes.issue(&player, &project, &step).await.expect("issue");
    assert_ne!(first, again, "a new code for the same step");
    assert_eq!(
        codes.current(&player, &project, &step).await.expect("current"),
        Some(again.clone()),
        "the new code replaces the old one (E10)",
    );

    let other_step = codes.issue(&player, &project, &fixtures::second_step()).await.expect("issue");
    assert_ne!(other_step, again, "steps get their own codes");

    let other_player = PlayerId::new("grace").expect("not blank");
    let theirs = codes.issue(&other_player, &project, &step).await.expect("issue");
    assert_ne!(theirs, again, "players get their own codes");
    assert_eq!(
        codes.current(&player, &project, &step).await.expect("current"),
        Some(again),
        "issuing for another step or player leaves this code valid",
    );
}

/// Issues `stamp-0`, `stamp-1`, …, so tests can predict them.
#[derive(Default)]
pub struct FakeStampCodes {
    issued: Mutex<Issued>,
}

#[derive(Default)]
struct Issued {
    count: u64,
    current: HashMap<(PlayerId, ProjectId, StepId), StampCode>,
}

impl FakeStampCodes {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl StampCodes for FakeStampCodes {
    async fn issue(
        &self,
        player: &PlayerId,
        project: &ProjectId,
        step: &StepId,
    ) -> Result<StampCode, PortError> {
        let mut issued = self.issued.lock().expect("no test panics while holding the lock");
        let code = StampCode::new(format!("stamp-{}", issued.count)).expect("not blank");
        issued.count += 1;
        issued.current.insert((player.clone(), project.clone(), step.clone()), code.clone());
        Ok(code)
    }

    async fn current(
        &self,
        player: &PlayerId,
        project: &ProjectId,
        step: &StepId,
    ) -> Result<Option<StampCode>, PortError> {
        let issued = self.issued.lock().expect("no test panics while holding the lock");
        Ok(issued.current.get(&(player.clone(), project.clone(), step.clone())).cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn the_fake_passes_the_suite() {
        suite(FakeStampCodes::new).await;
    }
}
