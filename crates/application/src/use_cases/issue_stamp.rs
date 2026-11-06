//! Hands the person a stamp for an approved step, when the stamp is on (M9, E10). Each new stamp
//! voids the previous one, and stamps don't expire.

use domain::player::PlayerId;
use domain::project::{CriterionId, ProjectId, StepId};
use domain::settings;
use domain::stamp::{StampCode, StampError};

use crate::ports::PortError;
use crate::ports::catalog::Catalog;
use crate::ports::progress::ProgressStore;
use crate::ports::settings::SettingsStore;
use crate::ports::stamp_codes::StampCodes;

#[derive(Debug, Clone)]
pub struct Input {
    pub player: PlayerId,
    pub project: ProjectId,
    pub step: StepId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output {
    pub code: StampCode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    UnknownProject(ProjectId),
    /// The player hasn't started this project.
    NotStarted(ProjectId),
    /// The project has no such step.
    UnknownStep(StepId),
    /// The step isn't approved yet (E2). `pending` lists the criteria that haven't passed.
    NotApproved {
        pending: Vec<CriterionId>,
    },
    /// The stamp is off, so the step was done at approval (E14).
    StampOff,
    Port(PortError),
}

pub async fn run(
    catalog: &dyn Catalog,
    progress_store: &dyn ProgressStore,
    settings_store: &dyn SettingsStore,
    stamp_codes: &dyn StampCodes,
    input: Input,
) -> Result<Output, Error> {
    let project = catalog
        .project(&input.project)
        .await
        .map_err(Error::Port)?
        .ok_or_else(|| Error::UnknownProject(input.project.clone()))?;
    let (progress, _) = progress_store
        .load(&input.player, &input.project)
        .await
        .map_err(Error::Port)?
        .ok_or_else(|| Error::NotStarted(input.project.clone()))?;
    let (game, project_settings, player_settings) =
        settings_store.load(&input.player, &input.project).await.map_err(Error::Port)?;
    let completion = settings::step_completion(&game, &project_settings, &player_settings);

    progress.stamp_allowed(&project, &input.step, completion).map_err(|error| match error {
        StampError::NotApproved { pending } => Error::NotApproved { pending },
        StampError::StampOff => Error::StampOff,
        StampError::UnknownStep(step) => Error::UnknownStep(step),
    })?;
    let code = stamp_codes.issue(&input.player, &input.project, &input.step).await.map_err(Error::Port)?;
    Ok(Output { code })
}

#[cfg(test)]
mod tests {
    use domain::progress::Progress;
    use domain::settings::GameSettings;
    use domain::stamp::StepCompletion;

    use super::*;
    use crate::testkit::catalog::FakeCatalog;
    use crate::testkit::fixtures;
    use crate::testkit::progress::FakeProgress;
    use crate::testkit::settings::{FakeSettings, StoredSettings};
    use crate::testkit::stamp_codes::FakeStampCodes;

    fn catalog() -> FakeCatalog {
        FakeCatalog::new(vec![fixtures::project()])
    }

    fn stamp(game: GameSettings) -> FakeSettings {
        FakeSettings::new(StoredSettings::game(game))
    }

    async fn saved(progress: &Progress) -> FakeProgress {
        let store = FakeProgress::new();
        store.save(&fixtures::player(), progress, None).await.expect("first save");
        store
    }

    /// The first step approved, waiting for its stamp.
    fn approved() -> Progress {
        let mut progress = fixtures::started();
        progress
            .submit(
                &fixtures::project(),
                fixtures::all_pass(&fixtures::first_step()),
                StepCompletion::AtStamp,
            )
            .expect("approved");
        progress
    }

    fn input(step: StepId) -> Input {
        Input { player: fixtures::player(), project: fixtures::project().id().clone(), step }
    }

    #[tokio::test]
    async fn an_approved_step_gets_a_stamp() {
        let codes = FakeStampCodes::new();
        let output = run(
            &catalog(),
            &saved(&approved()).await,
            &stamp(fixtures::stamp_on()),
            &codes,
            input(fixtures::first_step()),
        )
        .await
        .expect("a stamp");

        let current =
            codes.current(&fixtures::player(), fixtures::project().id(), &fixtures::first_step()).await;
        assert_eq!(current, Ok(Some(output.code)));
    }

    #[tokio::test]
    async fn a_new_stamp_voids_the_old_one() {
        let (store, settings, codes) =
            (saved(&approved()).await, stamp(fixtures::stamp_on()), FakeStampCodes::new());
        let first =
            run(&catalog(), &store, &settings, &codes, input(fixtures::first_step())).await.expect("a stamp");
        let second =
            run(&catalog(), &store, &settings, &codes, input(fixtures::first_step())).await.expect("a stamp");

        assert_ne!(first, second);
        let current =
            codes.current(&fixtures::player(), fixtures::project().id(), &fixtures::first_step()).await;
        assert_eq!(current, Ok(Some(second.code)));
    }

    #[tokio::test]
    async fn a_step_not_yet_approved_names_the_criteria_not_passed() {
        let codes = FakeStampCodes::new();
        let output = run(
            &catalog(),
            &saved(&fixtures::started()).await,
            &stamp(fixtures::stamp_on()),
            &codes,
            input(fixtures::first_step()),
        )
        .await;

        let project = fixtures::project();
        let step = project.step(&fixtures::first_step()).expect("the first step");
        let pending = step.criteria.iter().map(|criterion| criterion.id.clone()).collect();
        assert_eq!(output, Err(Error::NotApproved { pending }));
    }

    #[tokio::test]
    async fn no_stamp_is_issued_when_the_stamp_is_off() {
        let codes = FakeStampCodes::new();
        let output = run(
            &catalog(),
            &saved(&approved()).await,
            &stamp(fixtures::stamp_off()),
            &codes,
            input(fixtures::first_step()),
        )
        .await;

        assert_eq!(output, Err(Error::StampOff));
        let current =
            codes.current(&fixtures::player(), fixtures::project().id(), &fixtures::first_step()).await;
        assert_eq!(current, Ok(None), "no code was issued");
    }

    #[tokio::test]
    async fn an_unknown_step_is_refused() {
        let unknown = StepId::new("deploy").expect("not blank");
        let output = run(
            &catalog(),
            &saved(&approved()).await,
            &stamp(fixtures::stamp_on()),
            &FakeStampCodes::new(),
            input(unknown.clone()),
        )
        .await;

        assert_eq!(output, Err(Error::UnknownStep(unknown)));
    }

    #[tokio::test]
    async fn a_project_not_started_is_refused() {
        let output = run(
            &catalog(),
            &FakeProgress::new(),
            &stamp(fixtures::stamp_on()),
            &FakeStampCodes::new(),
            input(fixtures::first_step()),
        )
        .await;

        assert_eq!(output, Err(Error::NotStarted(fixtures::project().id().clone())));
    }
}
