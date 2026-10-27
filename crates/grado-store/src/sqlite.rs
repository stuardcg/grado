use std::str::FromStr;

use async_trait::async_trait;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions};
use uuid::Uuid;

use crate::{
    AccessToken, Attempt, Idea, NewAttempt, PlayerSummary, RedeemOutcome, Result, Store, Timestamp, User, now,
};

#[derive(Clone)]
pub struct SqliteStore {
    pool: SqlitePool,
}

impl SqliteStore {
    /// Opens (creating if needed) the database at `url`, e.g.
    /// `sqlite://grado.db` or `sqlite::memory:`, and applies migrations.
    pub async fn connect(url: &str) -> Result<Self> {
        let options = SqliteConnectOptions::from_str(url)?
            .create_if_missing(true)
            .foreign_keys(true)
            .journal_mode(SqliteJournalMode::Wal);
        // An in-memory database exists per connection, so keep exactly one.
        let max = if url.contains(":memory:") { 1 } else { 5 };
        let pool = SqlitePoolOptions::new().max_connections(max).connect_with(options).await?;
        sqlx::migrate!("./migrations/sqlite").run(&pool).await?;
        Ok(Self { pool })
    }
}

fn new_id() -> String {
    Uuid::new_v4().to_string()
}

#[async_trait]
impl Store for SqliteStore {
    async fn upsert_user(&self, email: &str, name: &str) -> Result<User> {
        let email = email.trim().to_lowercase();
        sqlx::query(
            "INSERT INTO users (id, email, name, created_at) VALUES (?, ?, ?, ?)
             ON CONFLICT (email) DO UPDATE SET name = excluded.name",
        )
        .bind(new_id())
        .bind(&email)
        .bind(name.trim())
        .bind(now())
        .execute(&self.pool)
        .await?;
        Ok(sqlx::query_as("SELECT * FROM users WHERE email = ?").bind(&email).fetch_one(&self.pool).await?)
    }

    async fn create_session(&self, user_id: &str, id_hash: &str, expires_at: Timestamp) -> Result<()> {
        sqlx::query("INSERT INTO sessions (id_hash, user_id, created_at, expires_at) VALUES (?, ?, ?, ?)")
            .bind(id_hash)
            .bind(user_id)
            .bind(now())
            .bind(expires_at)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn session_user(&self, id_hash: &str) -> Result<Option<User>> {
        Ok(sqlx::query_as(
            "SELECT u.* FROM sessions s JOIN users u ON u.id = s.user_id
             WHERE s.id_hash = ? AND s.expires_at > ?",
        )
        .bind(id_hash)
        .bind(now())
        .fetch_optional(&self.pool)
        .await?)
    }

    async fn delete_session(&self, id_hash: &str) -> Result<()> {
        sqlx::query("DELETE FROM sessions WHERE id_hash = ?").bind(id_hash).execute(&self.pool).await?;
        Ok(())
    }

    async fn create_access_token(&self, user_id: &str, label: &str, token_hash: &str) -> Result<AccessToken> {
        let id = new_id();
        sqlx::query(
            "INSERT INTO access_tokens (id, user_id, token_hash, label, created_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(user_id)
        .bind(token_hash)
        .bind(label)
        .bind(now())
        .execute(&self.pool)
        .await?;
        Ok(sqlx::query_as("SELECT id, label, created_at, last_used_at FROM access_tokens WHERE id = ?")
            .bind(&id)
            .fetch_one(&self.pool)
            .await?)
    }

    async fn access_tokens(&self, user_id: &str) -> Result<Vec<AccessToken>> {
        Ok(sqlx::query_as(
            "SELECT id, label, created_at, last_used_at FROM access_tokens
             WHERE user_id = ? AND revoked_at IS NULL ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?)
    }

    async fn revoke_access_token(&self, user_id: &str, token_id: &str) -> Result<bool> {
        let result = sqlx::query(
            "UPDATE access_tokens SET revoked_at = ? WHERE id = ? AND user_id = ? AND revoked_at IS NULL",
        )
        .bind(now())
        .bind(token_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn access_token_user(&self, token_hash: &str) -> Result<Option<User>> {
        let user: Option<User> = sqlx::query_as(
            "SELECT u.* FROM access_tokens t JOIN users u ON u.id = t.user_id
             WHERE t.token_hash = ? AND t.revoked_at IS NULL",
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await?;
        if user.is_some() {
            sqlx::query("UPDATE access_tokens SET last_used_at = ? WHERE token_hash = ?")
                .bind(now())
                .bind(token_hash)
                .execute(&self.pool)
                .await?;
        }
        Ok(user)
    }

    async fn completed_levels(&self, user_id: &str) -> Result<Vec<String>> {
        Ok(sqlx::query_scalar(
            "SELECT level_id FROM level_progress WHERE user_id = ? AND completed_at IS NOT NULL",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?)
    }

    async fn start_level(&self, user_id: &str, level_id: &str, content_version: &str) -> Result<()> {
        sqlx::query(
            "INSERT INTO level_progress (user_id, level_id, started_at, started_version) VALUES (?, ?, ?, ?)
             ON CONFLICT (user_id, level_id) DO NOTHING",
        )
        .bind(user_id)
        .bind(level_id)
        .bind(now())
        .bind(content_version)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn record_attempt(&self, attempt: NewAttempt<'_>) -> Result<()> {
        sqlx::query(
            "INSERT INTO criterion_attempts
               (id, user_id, level_id, criterion_id, evidence, passed, feedback, grader, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(new_id())
        .bind(attempt.user_id)
        .bind(attempt.level_id)
        .bind(attempt.criterion_id)
        .bind(attempt.evidence)
        .bind(attempt.passed)
        .bind(attempt.feedback)
        .bind(attempt.grader)
        .bind(now())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn passed_criteria(&self, user_id: &str, level_id: &str) -> Result<Vec<String>> {
        Ok(sqlx::query_scalar(
            "SELECT DISTINCT criterion_id FROM criterion_attempts
             WHERE user_id = ? AND level_id = ? AND passed = 1",
        )
        .bind(user_id)
        .bind(level_id)
        .fetch_all(&self.pool)
        .await?)
    }

    async fn attempts(&self, user_id: &str, level_id: Option<&str>) -> Result<Vec<Attempt>> {
        Ok(sqlx::query_as(
            "SELECT id, level_id, criterion_id, evidence, passed, feedback, grader, created_at
             FROM criterion_attempts
             WHERE user_id = ? AND (? IS NULL OR level_id = ?)
             ORDER BY created_at DESC",
        )
        .bind(user_id)
        .bind(level_id)
        .bind(level_id)
        .fetch_all(&self.pool)
        .await?)
    }

    async fn issue_stamp(
        &self,
        user_id: &str,
        level_id: &str,
        stamp_hash: &str,
        expires_at: Timestamp,
    ) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "UPDATE checkpoint_stamps SET voided_at = ?
             WHERE user_id = ? AND level_id = ? AND redeemed_at IS NULL AND voided_at IS NULL",
        )
        .bind(now())
        .bind(user_id)
        .bind(level_id)
        .execute(&mut *tx)
        .await?;
        sqlx::query(
            "INSERT INTO checkpoint_stamps (stamp_hash, user_id, level_id, created_at, expires_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(stamp_hash)
        .bind(user_id)
        .bind(level_id)
        .bind(now())
        .bind(expires_at)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(())
    }

    async fn redeem_stamp(
        &self,
        user_id: &str,
        level_id: &str,
        stamp_hash: &str,
        content_version: &str,
    ) -> Result<RedeemOutcome> {
        let mut tx = self.pool.begin().await?;
        let already: Option<i64> = sqlx::query_scalar(
            "SELECT completed_at FROM level_progress WHERE user_id = ? AND level_id = ? AND completed_at IS NOT NULL",
        )
        .bind(user_id)
        .bind(level_id)
        .fetch_optional(&mut *tx)
        .await?;
        if already.is_some() {
            return Ok(RedeemOutcome::AlreadyCompleted);
        }

        let stamp: Option<(i64, Option<i64>)> = sqlx::query_as(
            "SELECT expires_at, redeemed_at FROM checkpoint_stamps
             WHERE stamp_hash = ? AND user_id = ? AND level_id = ? AND voided_at IS NULL",
        )
        .bind(stamp_hash)
        .bind(user_id)
        .bind(level_id)
        .fetch_optional(&mut *tx)
        .await?;
        let Some((expires_at, redeemed_at)) = stamp else {
            return Ok(RedeemOutcome::Invalid);
        };
        if redeemed_at.is_some() {
            return Ok(RedeemOutcome::AlreadyCompleted);
        }
        let at = now();
        if expires_at <= at {
            return Ok(RedeemOutcome::Expired);
        }

        sqlx::query("UPDATE checkpoint_stamps SET redeemed_at = ? WHERE stamp_hash = ?")
            .bind(at)
            .bind(stamp_hash)
            .execute(&mut *tx)
            .await?;
        sqlx::query(
            "INSERT INTO level_progress (user_id, level_id, started_at, started_version, completed_at, completed_version)
             VALUES (?, ?, ?, ?, ?, ?)
             ON CONFLICT (user_id, level_id) DO UPDATE
               SET completed_at = excluded.completed_at, completed_version = excluded.completed_version",
        )
        .bind(user_id)
        .bind(level_id)
        .bind(at)
        .bind(content_version)
        .bind(at)
        .bind(content_version)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(RedeemOutcome::Redeemed)
    }

    async fn save_handoff(&self, user_id: &str, level_id: &str, body: &str) -> Result<()> {
        sqlx::query(
            "INSERT INTO handoffs (user_id, level_id, body, updated_at) VALUES (?, ?, ?, ?)
             ON CONFLICT (user_id, level_id) DO UPDATE SET body = excluded.body, updated_at = excluded.updated_at",
        )
        .bind(user_id)
        .bind(level_id)
        .bind(body)
        .bind(now())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn handoff(&self, user_id: &str, level_id: &str) -> Result<Option<String>> {
        Ok(sqlx::query_scalar("SELECT body FROM handoffs WHERE user_id = ? AND level_id = ?")
            .bind(user_id)
            .bind(level_id)
            .fetch_optional(&self.pool)
            .await?)
    }

    async fn unlock_guide(&self, user_id: &str, level_id: &str, tier: u8) -> Result<()> {
        sqlx::query(
            "INSERT INTO guide_unlocks (user_id, level_id, tier, unlocked_at) VALUES (?, ?, ?, ?)
             ON CONFLICT DO NOTHING",
        )
        .bind(user_id)
        .bind(level_id)
        .bind(tier as i64)
        .bind(now())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn guides_unlocked(&self, user_id: &str, level_id: &str) -> Result<u8> {
        let max: Option<i64> =
            sqlx::query_scalar("SELECT MAX(tier) FROM guide_unlocks WHERE user_id = ? AND level_id = ?")
                .bind(user_id)
                .bind(level_id)
                .fetch_one(&self.pool)
                .await?;
        Ok(max.unwrap_or(0) as u8)
    }

    async fn park_idea(
        &self,
        user_id: &str,
        raised_in: &str,
        belongs_to: Option<&str>,
        body: &str,
    ) -> Result<Idea> {
        let id = new_id();
        sqlx::query(
            "INSERT INTO ideas (id, user_id, raised_in_level, belongs_to_level, body, created_at)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(user_id)
        .bind(raised_in)
        .bind(belongs_to)
        .bind(body)
        .bind(now())
        .execute(&self.pool)
        .await?;
        Ok(sqlx::query_as(
            "SELECT id, raised_in_level, belongs_to_level, body, created_at FROM ideas WHERE id = ?",
        )
        .bind(&id)
        .fetch_one(&self.pool)
        .await?)
    }

    async fn ideas(&self, user_id: &str) -> Result<Vec<Idea>> {
        Ok(sqlx::query_as(
            "SELECT id, raised_in_level, belongs_to_level, body, created_at FROM ideas
             WHERE user_id = ? ORDER BY created_at",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?)
    }

    async fn players(&self) -> Result<Vec<PlayerSummary>> {
        Ok(sqlx::query_as(
            "SELECT u.id, u.email, u.name,
               (SELECT COUNT(*) FROM level_progress p WHERE p.user_id = u.id AND p.completed_at IS NOT NULL) AS levels_completed,
               (SELECT COUNT(*) FROM criterion_attempts a WHERE a.user_id = u.id) AS attempts,
               (SELECT MAX(created_at) FROM criterion_attempts a WHERE a.user_id = u.id) AS last_activity
             FROM users u ORDER BY u.created_at",
        )
        .fetch_all(&self.pool)
        .await?)
    }

    async fn user(&self, user_id: &str) -> Result<Option<User>> {
        Ok(sqlx::query_as("SELECT * FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn store() -> SqliteStore {
        SqliteStore::connect("sqlite::memory:").await.unwrap()
    }

    #[tokio::test]
    async fn stamp_redeems_once_and_completes_level() {
        let store = store().await;
        let user = store.upsert_user("Ana@Example.com", "Ana").await.unwrap();
        assert_eq!(user.email, "ana@example.com");

        store.start_level(&user.id, "00-speedrun", "v1").await.unwrap();
        store.issue_stamp(&user.id, "00-speedrun", "old", now() + 60).await.unwrap();
        store.issue_stamp(&user.id, "00-speedrun", "new", now() + 60).await.unwrap();

        assert_eq!(
            store.redeem_stamp(&user.id, "00-speedrun", "old", "v1").await.unwrap(),
            RedeemOutcome::Invalid
        );
        assert_eq!(
            store.redeem_stamp(&user.id, "00-speedrun", "new", "v1").await.unwrap(),
            RedeemOutcome::Redeemed
        );
        assert_eq!(
            store.redeem_stamp(&user.id, "00-speedrun", "new", "v1").await.unwrap(),
            RedeemOutcome::AlreadyCompleted
        );
        assert_eq!(store.completed_levels(&user.id).await.unwrap(), vec!["00-speedrun".to_string()]);
    }

    #[tokio::test]
    async fn expired_stamp_is_rejected() {
        let store = store().await;
        let user = store.upsert_user("ana@example.com", "Ana").await.unwrap();
        store.issue_stamp(&user.id, "00-speedrun", "s", now() - 1).await.unwrap();
        assert_eq!(
            store.redeem_stamp(&user.id, "00-speedrun", "s", "v1").await.unwrap(),
            RedeemOutcome::Expired
        );
    }

    #[tokio::test]
    async fn revoked_access_token_stops_resolving() {
        let store = store().await;
        let user = store.upsert_user("ana@example.com", "Ana").await.unwrap();
        let token = store.create_access_token(&user.id, "laptop", "h").await.unwrap();
        assert!(store.access_token_user("h").await.unwrap().is_some());
        assert!(store.revoke_access_token(&user.id, &token.id).await.unwrap());
        assert!(store.access_token_user("h").await.unwrap().is_none());
    }
}
