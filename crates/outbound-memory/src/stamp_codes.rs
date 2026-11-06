//! Stamp codes from random UUIDs, held in memory.

use std::collections::HashMap;
use std::sync::Mutex;

use application::ports::PortError;
use application::ports::stamp_codes::StampCodes;
use async_trait::async_trait;
use domain::player::PlayerId;
use domain::project::{ProjectId, StepId};
use domain::stamp::StampCode;
use uuid::Uuid;

use crate::lock;

#[derive(Default)]
pub struct MemoryStampCodes {
    current: Mutex<HashMap<(PlayerId, ProjectId, StepId), StampCode>>,
}

impl MemoryStampCodes {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl StampCodes for MemoryStampCodes {
    async fn issue(
        &self,
        player: &PlayerId,
        project: &ProjectId,
        step: &StepId,
    ) -> Result<StampCode, PortError> {
        let code = StampCode::new(Uuid::new_v4().to_string()).expect("a UUID isn't blank");
        let mut current = lock(&self.current)?;
        current.insert((player.clone(), project.clone(), step.clone()), code.clone());
        Ok(code)
    }

    async fn current(
        &self,
        player: &PlayerId,
        project: &ProjectId,
        step: &StepId,
    ) -> Result<Option<StampCode>, PortError> {
        let current = lock(&self.current)?;
        Ok(current.get(&(player.clone(), project.clone(), step.clone())).cloned())
    }
}

#[cfg(test)]
mod tests {
    use application::testkit::stamp_codes::suite;

    use super::MemoryStampCodes;

    #[tokio::test]
    async fn passes_the_stamp_codes_suite() {
        suite(MemoryStampCodes::new).await;
    }
}
