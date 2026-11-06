//! Records an agent's pass or fail per criterion on the open step (M8, M9). A fail is recorded
//! and leaves the step open; an all-pass report with a handoff note approves the step, and with
//! the stamp off also completes it and opens the next one (M11).

use domain::player::PlayerId;
use domain::project::{CriterionId, ProjectId, StepId};
use domain::report::{SubmitError, SubmitReport, Submitted};
use domain::settings;

use crate::ports::PortError;
use crate::ports::catalog::Catalog;
use crate::ports::progress::{ProgressStore, SaveProgressError};
use crate::ports::settings::SettingsStore;

#[derive(Debug, Clone)]
pub struct Input {
    pub player: PlayerId,
    pub project: ProjectId,
    pub report: SubmitReport,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output {
    pub submitted: Submitted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    UnknownProject(ProjectId),
    /// The player hasn't started this project.
    NotStarted(ProjectId),
    /// The project has no such step.
    UnknownStep(StepId),
    /// The report isn't about the open step (E1). `open` is `None` when no step is open.
    StepNotOpen {
        open: Option<StepId>,
    },
    /// The verdicts don't cover exactly the step's criteria.
    CriteriaMismatch {
        expected: Vec<CriterionId>,
    },
    /// Every criterion passed, but there is no handoff note (ADR 0006 #11).
    HandoffNoteMissing,
    /// Another connection saved this progress since it was loaded (E5); nothing was recorded.
    Conflict,
    Port(PortError),
}

pub async fn run(
    catalog: &dyn Catalog,
    progress_store: &dyn ProgressStore,
    settings_store: &dyn SettingsStore,
    input: Input,
) -> Result<Output, Error> {
    let project = catalog
        .project(&input.project)
        .await
        .map_err(Error::Port)?
        .ok_or_else(|| Error::UnknownProject(input.project.clone()))?;
    let (mut progress, version) = progress_store
        .load(&input.player, &input.project)
        .await
        .map_err(Error::Port)?
        .ok_or_else(|| Error::NotStarted(input.project.clone()))?;
    let (game, project_settings, player_settings) =
        settings_store.load(&input.player, &input.project).await.map_err(Error::Port)?;
    let completion = settings::step_completion(&game, &project_settings, &player_settings);

    let submitted = progress.submit(&project, input.report, completion).map_err(|error| match error {
        SubmitError::UnknownStep(step) => Error::UnknownStep(step),
        SubmitError::StepNotOpen { open } => Error::StepNotOpen { open },
        SubmitError::CriteriaMismatch { expected } => Error::CriteriaMismatch { expected },
        SubmitError::HandoffNoteMissing => Error::HandoffNoteMissing,
    })?;
    progress_store.save(&input.player, &progress, Some(version)).await.map_err(|error| match error {
        SaveProgressError::Conflict => Error::Conflict,
        SaveProgressError::Port(error) => Error::Port(error),
    })?;
    Ok(Output { submitted })
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use domain::progress::{Progress, StepState};
    use domain::settings::{GameSettings, PlayerSettings};
    use domain::stamp::StampSetting;

    use super::*;
    use crate::ports::progress::Version;
    use crate::testkit::catalog::FakeCatalog;
    use crate::testkit::fixtures;
    use crate::testkit::progress::FakeProgress;
    use crate::testkit::settings::{FakeSettings, StoredSettings};

    fn catalog() -> FakeCatalog {
        FakeCatalog::new(vec![fixtures::project()])
    }

    fn stamp(game: GameSettings) -> FakeSettings {
        FakeSettings::new(StoredSettings::game(game))
    }

    async fn started() -> FakeProgress {
        let store = FakeProgress::new();
        store.save(&fixtures::player(), &fixtures::started(), None).await.expect("first save");
        store
    }

    fn input(report: SubmitReport) -> Input {
        Input { player: fixtures::player(), project: fixtures::project().id().clone(), report }
    }

    async fn saved(store: &FakeProgress) -> Progress {
        let (progress, _) =
            store.load(&fixtures::player(), fixtures::project().id()).await.expect("load").expect("started");
        progress
    }

    #[tokio::test]
    async fn all_pass_with_the_stamp_off_completes_the_step_and_opens_the_next() {
        let store = started().await;
        let stamp_off = stamp(fixtures::stamp_off());
        let report = fixtures::all_pass(&fixtures::first_step());

        let output = run(&catalog(), &store, &stamp_off, input(report)).await;

        assert_eq!(
            output,
            Ok(Output { submitted: Submitted::StepDone { next: Some(fixtures::second_step()) } })
        );
        let progress = saved(&store).await;
        assert_eq!(progress.state(&fixtures::first_step()), Some(StepState::Done));
        assert_eq!(progress.state(&fixtures::second_step()), Some(StepState::Open));
    }

    #[tokio::test]
    async fn all_pass_with_the_stamp_on_stops_at_approved() {
        let store = started().await;
        let stamp_on = stamp(fixtures::stamp_on());
        let report = fixtures::all_pass(&fixtures::first_step());

        let output = run(&catalog(), &store, &stamp_on, input(report)).await;

        assert_eq!(output, Ok(Output { submitted: Submitted::StepApproved }));
        assert_eq!(saved(&store).await.state(&fixtures::first_step()), Some(StepState::Approved));
    }

    #[tokio::test]
    async fn the_player_setting_wins_over_the_game() {
        let store = started().await;
        let player_off = FakeSettings::new(StoredSettings {
            game: fixtures::stamp_on(),
            projects: Vec::new(),
            players: vec![(fixtures::player(), PlayerSettings { stamp: Some(StampSetting::Off) })],
        });
        let report = fixtures::all_pass(&fixtures::first_step());

        let output = run(&catalog(), &store, &player_off, input(report)).await;

        assert_eq!(
            output,
            Ok(Output { submitted: Submitted::StepDone { next: Some(fixtures::second_step()) } })
        );
    }

    #[tokio::test]
    async fn a_fail_is_recorded_and_names_the_failed_criteria() {
        let store = started().await;
        let report = fixtures::all_fail(&fixtures::first_step());

        let output = run(&catalog(), &store, &stamp(fixtures::stamp_on()), input(report.clone())).await;

        let failed = report.verdicts.iter().map(|(criterion, _)| criterion.clone()).collect();
        assert_eq!(output, Ok(Output { submitted: Submitted::StepStillOpen { failed } }));
        let progress = saved(&store).await;
        assert_eq!(progress.state(&fixtures::first_step()), Some(StepState::Open));
        assert!(progress.latest_report(&fixtures::first_step()).is_some(), "the fail is recorded");
    }

    #[tokio::test]
    async fn a_report_on_a_step_that_isnt_open_names_the_open_step() {
        let store = started().await;
        let report = fixtures::all_pass(&fixtures::second_step());

        let output = run(&catalog(), &store, &stamp(fixtures::stamp_on()), input(report)).await;

        assert_eq!(output, Err(Error::StepNotOpen { open: Some(fixtures::first_step()) }));
        assert_eq!(saved(&store).await, fixtures::started(), "nothing is saved");
    }

    #[tokio::test]
    async fn all_pass_without_a_note_is_refused() {
        let store = started().await;
        let report = SubmitReport { handoff_note: None, ..fixtures::all_pass(&fixtures::first_step()) };

        let output = run(&catalog(), &store, &stamp(fixtures::stamp_on()), input(report)).await;

        assert_eq!(output, Err(Error::HandoffNoteMissing));
    }

    #[tokio::test]
    async fn a_project_not_started_is_refused() {
        let report = fixtures::all_pass(&fixtures::first_step());
        let stamp_on = stamp(fixtures::stamp_on());

        let output = run(&catalog(), &FakeProgress::new(), &stamp_on, input(report)).await;

        assert_eq!(output, Err(Error::NotStarted(fixtures::project().id().clone())));
    }

    /// Lets another connection save right after this one loads.
    struct Racing(FakeProgress);

    #[async_trait]
    impl ProgressStore for Racing {
        async fn started(&self, player: &PlayerId) -> Result<Vec<ProjectId>, PortError> {
            self.0.started(player).await
        }

        async fn load(
            &self,
            player: &PlayerId,
            project: &ProjectId,
        ) -> Result<Option<(Progress, Version)>, PortError> {
            let loaded = self.0.load(player, project).await?;
            if let Some((progress, version)) = &loaded {
                self.0.save(player, progress, Some(*version)).await.expect("the other connection saves");
            }
            Ok(loaded)
        }

        async fn save(
            &self,
            player: &PlayerId,
            progress: &Progress,
            loaded: Option<Version>,
        ) -> Result<Version, SaveProgressError> {
            self.0.save(player, progress, loaded).await
        }
    }

    #[tokio::test]
    async fn a_save_that_lost_a_race_is_refused() {
        let store = Racing(started().await);
        let report = fixtures::all_pass(&fixtures::first_step());

        let output = run(&catalog(), &store, &stamp(fixtures::stamp_off()), input(report)).await;

        assert_eq!(output, Err(Error::Conflict));
        assert_eq!(saved(&store.0).await, fixtures::started(), "the other connection's save stands");
    }
}
