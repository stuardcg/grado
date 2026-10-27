//! Persistence for Grado.
//!
//! The rest of the system talks to [`Store`] only. [`SqliteStore`] is the v1
//! implementation; a Postgres one implements the same trait with its own
//! migrations under `migrations/postgres/`.

mod sqlite;

use async_trait::async_trait;
use serde::Serialize;

pub use sqlite::SqliteStore;

pub type Timestamp = i64;

pub fn now() -> Timestamp {
    time::OffsetDateTime::now_utc().unix_timestamp()
}

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Migration(#[from] sqlx::migrate::MigrateError),
}

pub type Result<T> = std::result::Result<T, StoreError>;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: String,
    pub created_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct AccessToken {
    pub id: String,
    pub label: String,
    pub created_at: Timestamp,
    pub last_used_at: Option<Timestamp>,
}

#[derive(Debug, Clone)]
pub struct NewAttempt<'a> {
    pub user_id: &'a str,
    pub level_id: &'a str,
    pub criterion_id: &'a str,
    pub evidence: &'a str,
    pub passed: bool,
    pub feedback: &'a str,
    pub grader: &'a str,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Attempt {
    pub id: String,
    pub level_id: String,
    pub criterion_id: String,
    pub evidence: String,
    pub passed: bool,
    pub feedback: String,
    pub grader: String,
    pub created_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Idea {
    pub id: String,
    pub raised_in_level: String,
    pub belongs_to_level: Option<String>,
    pub body: String,
    pub created_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct PlayerSummary {
    pub id: String,
    pub email: String,
    pub name: String,
    pub levels_completed: i64,
    pub attempts: i64,
    pub last_activity: Option<Timestamp>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RedeemOutcome {
    Redeemed,
    AlreadyCompleted,
    Expired,
    Invalid,
}

#[async_trait]
pub trait Store: Send + Sync {
    // Players and sign-in
    async fn upsert_user(&self, email: &str, name: &str) -> Result<User>;
    async fn create_session(&self, user_id: &str, id_hash: &str, expires_at: Timestamp) -> Result<()>;
    async fn session_user(&self, id_hash: &str) -> Result<Option<User>>;
    async fn delete_session(&self, id_hash: &str) -> Result<()>;

    // Claude Code access tokens
    async fn create_access_token(&self, user_id: &str, label: &str, token_hash: &str) -> Result<AccessToken>;
    async fn access_tokens(&self, user_id: &str) -> Result<Vec<AccessToken>>;
    async fn revoke_access_token(&self, user_id: &str, token_id: &str) -> Result<bool>;
    /// Resolves a bearer token and records that it was used.
    async fn access_token_user(&self, token_hash: &str) -> Result<Option<User>>;

    // Progress
    async fn completed_levels(&self, user_id: &str) -> Result<Vec<String>>;
    async fn start_level(&self, user_id: &str, level_id: &str, content_version: &str) -> Result<()>;
    async fn record_attempt(&self, attempt: NewAttempt<'_>) -> Result<()>;
    async fn passed_criteria(&self, user_id: &str, level_id: &str) -> Result<Vec<String>>;
    async fn attempts(&self, user_id: &str, level_id: Option<&str>) -> Result<Vec<Attempt>>;

    // Checkpoint stamps
    /// Stores a new stamp for the level and voids any earlier unredeemed one.
    async fn issue_stamp(
        &self,
        user_id: &str,
        level_id: &str,
        stamp_hash: &str,
        expires_at: Timestamp,
    ) -> Result<()>;
    /// Redeems the stamp and completes the level in one transaction.
    async fn redeem_stamp(
        &self,
        user_id: &str,
        level_id: &str,
        stamp_hash: &str,
        content_version: &str,
    ) -> Result<RedeemOutcome>;

    // Handoffs, guides, ideas
    async fn save_handoff(&self, user_id: &str, level_id: &str, body: &str) -> Result<()>;
    async fn handoff(&self, user_id: &str, level_id: &str) -> Result<Option<String>>;
    async fn unlock_guide(&self, user_id: &str, level_id: &str, tier: u8) -> Result<()>;
    async fn guides_unlocked(&self, user_id: &str, level_id: &str) -> Result<u8>;
    async fn park_idea(
        &self,
        user_id: &str,
        raised_in: &str,
        belongs_to: Option<&str>,
        body: &str,
    ) -> Result<Idea>;
    async fn ideas(&self, user_id: &str) -> Result<Vec<Idea>>;

    // Game master
    async fn players(&self) -> Result<Vec<PlayerSummary>>;
    async fn user(&self, user_id: &str) -> Result<Option<User>>;
}
