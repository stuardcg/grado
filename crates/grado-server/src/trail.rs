//! The game rules as operations on one player. Both the website and the MCP
//! endpoint go through [`Trail`], so a rule lives in exactly one place.

use std::sync::Arc;

use serde::Serialize;
use grado_domain::briefing::{self, BriefingState};
use grado_domain::{Catalog, Criterion, EvidenceKind, Journey, Level, LevelStatus, RubricGrader, secrets};
use grado_store::{Idea, NewAttempt, RedeemOutcome, Store, StoreError, User, now};

use crate::markdown;

#[derive(Debug, thiserror::Error)]
pub enum TrailError {
    #[error("{0}")]
    NotFound(String),
    /// The request is understood but not allowed right now. The message is
    /// written for the coaching agent or the player to act on.
    #[error("{0}")]
    NotYet(String),
    #[error("{0}")]
    Invalid(String),
    #[error(transparent)]
    Store(#[from] StoreError),
}

pub type Result<T> = std::result::Result<T, TrailError>;

pub struct Trail {
    pub catalog: Arc<Catalog>,
    pub store: Arc<dyn Store>,
    grader: Arc<dyn RubricGrader>,
    stamp_ttl_secs: i64,
}

#[derive(Debug, Serialize)]
pub struct CheckView {
    pub level_id: String,
    pub criterion_id: String,
    pub position: usize,
    pub total: usize,
    pub title: String,
    pub ask: String,
    pub evidence: EvidenceKind,
}

#[derive(Debug)]
pub enum CurrentCheck {
    Pending(CheckView),
    AllPassed { level_id: String },
    Finished,
}

#[derive(Debug, Serialize)]
pub struct SubmitOutcome {
    pub passed: bool,
    pub feedback: String,
    pub checks_passed: usize,
    pub checks_total: usize,
}

#[derive(Debug, Serialize)]
pub struct Stamp {
    pub stamp: String,
    pub level_title: String,
    pub expires_in_hours: i64,
}

#[derive(Debug, Serialize)]
pub struct LevelSummary {
    pub id: String,
    pub number: u32,
    pub title: String,
    pub environment: grado_domain::Environment,
    pub environment_label: &'static str,
    pub habit: String,
    pub status: LevelStatus,
}

#[derive(Debug, Serialize)]
pub struct GuideView {
    pub tier: u8,
    pub title: String,
    /// `None` until unlocked.
    pub html: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LevelView {
    #[serde(flatten)]
    pub summary: LevelSummary,
    pub objective: String,
    pub produces: String,
    pub estimated_minutes: Option<u32>,
    pub page_html: String,
    pub checks_passed: usize,
    pub checks_total: usize,
    /// Titles of passed checks only; upcoming checks stay hidden.
    pub passed_titles: Vec<String>,
    pub ready_for_stamp: bool,
    pub guides: Vec<GuideView>,
    pub handoff: Option<String>,
}

impl Trail {
    pub fn new(
        catalog: Arc<Catalog>,
        store: Arc<dyn Store>,
        grader: Arc<dyn RubricGrader>,
        stamp_ttl_hours: i64,
    ) -> Self {
        Self { catalog, store, grader, stamp_ttl_secs: stamp_ttl_hours * 3600 }
    }

    async fn journey(&self, user: &User) -> Result<Journey<'_>> {
        let completed = self.store.completed_levels(&user.id).await?;
        Ok(Journey::new(&self.catalog, completed))
    }

    /// The active level, recording that the player has started it.
    pub async fn current_level(&self, user: &User) -> Result<Option<&Level>> {
        let journey = self.journey(user).await?;
        let Some(level) = journey.current() else { return Ok(None) };
        self.store.start_level(&user.id, &level.id, &self.catalog.version).await?;
        Ok(Some(level))
    }

    async fn require_current(&self, user: &User) -> Result<&Level> {
        self.current_level(user).await?.ok_or_else(|| {
            TrailError::NotYet("She has finished every checkpoint. Nothing left to submit.".into())
        })
    }

    async fn passed(&self, user: &User, level: &Level) -> Result<Vec<String>> {
        Ok(self.store.passed_criteria(&user.id, &level.id).await?)
    }

    fn first_unpassed<'a>(level: &'a Level, passed: &[String]) -> Option<(usize, &'a Criterion)> {
        level.spec.criteria.iter().enumerate().find(|(_, c)| !passed.contains(&c.id))
    }

    pub async fn briefing(&self, user: &User) -> Result<String> {
        let Some(level) = self.current_level(user).await? else {
            return Ok(format!(
                "{}\n\n---\n\n# Summit reached\n\n{} has completed every checkpoint. Congratulate her, and if she wants to keep going, help her pick one of her parked ideas as a new adventure.",
                self.catalog.core_md, user.name
            ));
        };
        let previous_handoff = match self.catalog.previous_level(&level.id) {
            Some(prev) => self.store.handoff(&user.id, &prev.id).await?,
            None => None,
        };
        let parked_ideas = self
            .store
            .ideas(&user.id)
            .await?
            .into_iter()
            .filter(|idea| idea.belongs_to_level.as_deref() == Some(level.id.as_str()))
            .map(|idea| idea.body)
            .collect();
        let state = BriefingState {
            player_name: user.name.clone(),
            previous_handoff,
            checks_passed: self.passed(user, level).await?.len(),
            guides_unlocked: self.store.guides_unlocked(&user.id, &level.id).await?,
            parked_ideas,
        };
        Ok(briefing::render(&self.catalog, level, &state))
    }

    pub async fn current_check(&self, user: &User) -> Result<CurrentCheck> {
        let Some(level) = self.current_level(user).await? else { return Ok(CurrentCheck::Finished) };
        let passed = self.passed(user, level).await?;
        Ok(match Self::first_unpassed(level, &passed) {
            Some((index, criterion)) => CurrentCheck::Pending(CheckView {
                level_id: level.id.clone(),
                criterion_id: criterion.id.clone(),
                position: index + 1,
                total: level.spec.criteria.len(),
                title: criterion.title.clone(),
                ask: criterion.ask.trim().to_owned(),
                evidence: criterion.evidence,
            }),
            None => CurrentCheck::AllPassed { level_id: level.id.clone() },
        })
    }

    pub async fn submit_evidence(
        &self,
        user: &User,
        criterion_id: &str,
        evidence: &str,
    ) -> Result<SubmitOutcome> {
        let level = self.require_current(user).await?;
        let passed = self.passed(user, level).await?;
        let Some((_, current)) = Self::first_unpassed(level, &passed) else {
            return Err(TrailError::NotYet(
                "Every check for this checkpoint has passed. Write the handoff note with her and call finish_checkpoint.".into(),
            ));
        };
        if current.id != criterion_id {
            return Err(TrailError::NotYet(format!(
                "Only the current check can be submitted. The current check is `{}`; call get_current_check to see it.",
                current.id
            )));
        }
        if evidence.trim().is_empty() {
            return Err(TrailError::Invalid("Evidence is empty.".into()));
        }

        let verdict = grado_domain::evaluate(level, current, evidence, self.grader.as_ref()).await;
        self.store
            .record_attempt(NewAttempt {
                user_id: &user.id,
                level_id: &level.id,
                criterion_id: &current.id,
                evidence,
                passed: verdict.passed,
                feedback: &verdict.feedback,
                grader: &verdict.grader,
            })
            .await?;
        let checks_passed = passed.len() + usize::from(verdict.passed);
        Ok(SubmitOutcome {
            passed: verdict.passed,
            feedback: verdict.feedback,
            checks_passed,
            checks_total: level.spec.criteria.len(),
        })
    }

    /// Saves the handoff and issues a fresh stamp once every check passed.
    pub async fn finish_checkpoint(&self, user: &User, handoff: &str) -> Result<Stamp> {
        let level = self.require_current(user).await?;
        let passed = self.passed(user, level).await?;
        if let Some((_, pending)) = Self::first_unpassed(level, &passed) {
            return Err(TrailError::NotYet(format!(
                "Not every check has passed yet. Still open: `{}` ({}).",
                pending.id, pending.title
            )));
        }
        if handoff.trim().chars().count() < 40 {
            return Err(TrailError::Invalid(
                "The handoff note is too short. Write 5–8 lines with her: what she built, where it lives, decisions, open questions.".into(),
            ));
        }
        self.store.save_handoff(&user.id, &level.id, handoff.trim()).await?;
        let stamp = secrets::checkpoint_stamp();
        self.store
            .issue_stamp(&user.id, &level.id, &secrets::hash(&stamp), now() + self.stamp_ttl_secs)
            .await?;
        Ok(Stamp {
            stamp,
            level_title: level.spec.title.clone(),
            expires_in_hours: self.stamp_ttl_secs / 3600,
        })
    }

    pub async fn redeem_stamp(&self, user: &User, level_id: &str, stamp: &str) -> Result<RedeemOutcome> {
        let journey = self.journey(user).await?;
        if journey.status(level_id) == LevelStatus::Locked {
            return Err(TrailError::NotYet("That checkpoint isn't open yet.".into()));
        }
        Ok(self.store.redeem_stamp(&user.id, level_id, &secrets::hash(stamp), &self.catalog.version).await?)
    }

    /// Opens the next guide tier for a visible level and returns it.
    pub async fn open_next_guide(&self, user: &User, level_id: &str) -> Result<Option<&grado_domain::Guide>> {
        let level = self.visible_level(user, level_id).await?;
        let unlocked = self.store.guides_unlocked(&user.id, &level.id).await?;
        let Some(guide) = level.guides.iter().find(|g| g.tier == unlocked + 1) else { return Ok(None) };
        self.store.unlock_guide(&user.id, &level.id, guide.tier).await?;
        Ok(Some(guide))
    }

    pub async fn park_idea(&self, user: &User, idea: &str, belongs_to_number: Option<u32>) -> Result<Idea> {
        let raised_in = match self.current_level(user).await? {
            Some(level) => level.id.clone(),
            None => "summit".to_owned(),
        };
        let belongs_to = match belongs_to_number {
            Some(n) => Some(
                self.catalog
                    .levels()
                    .iter()
                    .find(|l| l.number() == n)
                    .ok_or_else(|| TrailError::Invalid(format!("There is no checkpoint {n}.")))?
                    .id
                    .clone(),
            ),
            None => None,
        };
        if idea.trim().is_empty() {
            return Err(TrailError::Invalid("The idea is empty.".into()));
        }
        Ok(self.store.park_idea(&user.id, &raised_in, belongs_to.as_deref(), idea.trim()).await?)
    }

    pub async fn journey_summary(&self, user: &User) -> Result<Vec<LevelSummary>> {
        let journey = self.journey(user).await?;
        Ok(self
            .catalog
            .levels()
            .iter()
            .map(|level| Self::summary(level, journey.status(&level.id)))
            .collect())
    }

    async fn visible_level(&self, user: &User, level_id: &str) -> Result<&Level> {
        let level = self
            .catalog
            .level(level_id)
            .ok_or_else(|| TrailError::NotFound(format!("No checkpoint called {level_id}.")))?;
        if !self.journey(user).await?.is_visible(level_id) {
            return Err(TrailError::NotYet("That checkpoint isn't open yet.".into()));
        }
        Ok(level)
    }

    pub async fn level_view(&self, user: &User, level_id: &str) -> Result<LevelView> {
        let level = self.visible_level(user, level_id).await?;
        let status = self.journey(user).await?.status(level_id);
        let passed = self.passed(user, level).await?;
        let unlocked = self.store.guides_unlocked(&user.id, &level.id).await?;
        let passed_titles =
            level.spec.criteria.iter().filter(|c| passed.contains(&c.id)).map(|c| c.title.clone()).collect();
        let guides = level
            .guides
            .iter()
            .map(|g| GuideView {
                tier: g.tier,
                title: g.title.clone(),
                html: (g.tier <= unlocked).then(|| markdown::to_html(&g.body_md)),
            })
            .collect();
        let handoff = if status == LevelStatus::Completed {
            self.store.handoff(&user.id, &level.id).await?
        } else {
            None
        };
        Ok(LevelView {
            summary: Self::summary(level, status),
            objective: level.spec.objective.clone(),
            produces: markdown::inline_to_html(&level.spec.produces),
            estimated_minutes: level.spec.estimated_minutes,
            page_html: markdown::to_html(&level.page_md),
            checks_passed: passed.len(),
            checks_total: level.spec.criteria.len(),
            passed_titles,
            ready_for_stamp: status == LevelStatus::Active && passed.len() == level.spec.criteria.len(),
            guides,
            handoff,
        })
    }

    fn summary(level: &Level, status: LevelStatus) -> LevelSummary {
        LevelSummary {
            id: level.id.clone(),
            number: level.number(),
            title: level.spec.title.clone(),
            environment: level.spec.environment,
            environment_label: level.spec.environment.label(),
            habit: level.spec.habit.clone(),
            status,
        }
    }
}
