//! Plays the real content through the service layer, with an in-memory
//! database and the offline grader.

use std::path::PathBuf;
use std::sync::Arc;

use grado_domain::{Catalog, LevelStatus, OfflineRubricGrader};
use grado_server::trail::{CurrentCheck, Trail, TrailError};
use grado_store::{RedeemOutcome, SqliteStore, Store, User};

async fn setup() -> (Trail, User) {
    let content = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../content");
    let catalog = Arc::new(Catalog::load(content).unwrap());
    let store = Arc::new(SqliteStore::connect("sqlite::memory:").await.unwrap());
    let user = store.upsert_user("player@example.com", "Player").await.unwrap();
    (Trail::new(catalog, store, Arc::new(OfflineRubricGrader), 24), user)
}

fn pending(check: CurrentCheck) -> String {
    match check {
        CurrentCheck::Pending(view) => view.criterion_id,
        other => panic!("expected a pending check, got {other:?}"),
    }
}

#[tokio::test]
async fn speedrun_checkpoint_end_to_end() {
    let (trail, user) = setup().await;

    let journey = trail.journey_summary(&user).await.unwrap();
    assert_eq!(journey[0].status, LevelStatus::Active);
    assert!(journey[1..].iter().all(|l| l.status == LevelStatus::Locked));

    // Checks come one at a time and out-of-order submissions are refused.
    assert_eq!(pending(trail.current_check(&user).await.unwrap()), "speedrun-saved");
    let out_of_order = trail.submit_evidence(&user, "first-impressions", "anything").await;
    assert!(matches!(out_of_order, Err(TrailError::NotYet(_))));

    assert!(!trail.submit_evidence(&user, "speedrun-saved", "README.md").await.unwrap().passed);
    assert!(trail.submit_evidence(&user, "speedrun-saved", "speedrun:\napp.py").await.unwrap().passed);

    // No stamp before every check passes.
    assert!(matches!(
        trail.finish_checkpoint(&user, "A handoff note that is long enough to be accepted.").await,
        Err(TrailError::NotYet(_))
    ));

    let impressions = "Worked: adding riders was easy. Felt off: data was gone after closing. \
                       I would not know how to change the categories.";
    assert!(trail.submit_evidence(&user, "first-impressions", impressions).await.unwrap().passed);
    assert!(matches!(trail.current_check(&user).await.unwrap(), CurrentCheck::AllPassed { .. }));

    let handoff = "Prompt: build me a registration app.\nApp in speedrun/.\nImpressions noted.";
    let stamp = trail.finish_checkpoint(&user, handoff).await.unwrap().stamp;

    // The next level can't be redeemed, and a wrong stamp doesn't work.
    assert!(trail.redeem_stamp(&user, "01-investigation", &stamp).await.is_err());
    assert_eq!(
        trail.redeem_stamp(&user, "00-speedrun", "STAMP-0000-0000").await.unwrap(),
        RedeemOutcome::Invalid
    );
    assert_eq!(
        trail.redeem_stamp(&user, "00-speedrun", &stamp.to_lowercase()).await.unwrap(),
        RedeemOutcome::Redeemed
    );

    // The handoff reaches the next briefing.
    let briefing = trail.briefing(&user).await.unwrap();
    assert!(briefing.contains("# Checkpoint 1: Investigation"));
    assert!(briefing.contains("Prompt: build me a registration app."));
    assert_eq!(pending(trail.current_check(&user).await.unwrap()), "users-and-moments");
}

#[tokio::test]
async fn locked_levels_and_guides_stay_hidden() {
    let (trail, user) = setup().await;
    assert!(matches!(trail.level_view(&user, "05-prototype").await, Err(TrailError::NotYet(_))));

    let view = trail.level_view(&user, "00-speedrun").await.unwrap();
    assert!(view.guides.iter().all(|g| g.html.is_none()));
    let first = trail.open_next_guide(&user, "00-speedrun").await.unwrap().unwrap();
    assert_eq!(first.tier, 1);
    let view = trail.level_view(&user, "00-speedrun").await.unwrap();
    assert!(view.guides[0].html.is_some() && view.guides[1].html.is_none());
}

#[tokio::test]
async fn prototype_summary_is_checked_against_hidden_values() {
    let (trail, _) = setup().await;
    let level = trail.catalog.level("05-prototype").unwrap();
    let criterion = level.criterion("summary").unwrap();
    let grader = OfflineRubricGrader;

    let right = "Kids: 5\nJunior: 5\nElite: 13\nMaster: 7\nRejected: 7\nDuplicates: 2\n";
    assert!(grado_domain::evaluate(level, criterion, right, &grader).await.passed);

    // The classic mistake: treating a rejected row as a duplicate.
    let wrong = "Kids: 5\nJunior: 5\nElite: 13\nMaster: 7\nRejected: 6\nDuplicates: 3\n";
    let verdict = grado_domain::evaluate(level, criterion, wrong, &grader).await;
    assert!(!verdict.passed);
    assert!(verdict.feedback.contains("Rejected") && verdict.feedback.contains("Duplicates"));
}
