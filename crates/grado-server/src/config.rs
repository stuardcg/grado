//! Settings from the environment. Every value has a local-development default
//! so `cargo run` works from a fresh clone.

use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub bind: String,
    /// Public base URL, used in briefings and in the MCP connect command.
    pub public_url: String,
    pub database_url: String,
    pub content_dir: PathBuf,
    pub web_dir: PathBuf,
    pub auth_mode: AuthMode,
    /// Emails allowed to see the game master view.
    pub admin_emails: Vec<String>,
    pub stamp_ttl_hours: i64,
    pub session_ttl_days: i64,
    /// Hostnames the MCP endpoint accepts in the `Host` header.
    pub allowed_hosts: Vec<String>,
    pub grader: GraderConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthMode {
    /// Sign in with just a name and email. Local development only.
    Dev,
}

#[derive(Debug, Clone)]
pub struct GraderConfig {
    /// When unset, rubric checks use the offline keyword grader.
    pub anthropic_api_key: Option<String>,
    pub model: String,
    pub effort: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let bind = var("GRADO_BIND", "127.0.0.1:8787");
        let public_url = var("GRADO_PUBLIC_URL", &format!("http://{bind}"));
        let auth_mode = match var("GRADO_AUTH_MODE", "dev").as_str() {
            "dev" => AuthMode::Dev,
            other => anyhow::bail!("GRADO_AUTH_MODE={other} is not supported yet; use `dev`"),
        };
        let mut allowed_hosts: Vec<String> = list("GRADO_ALLOWED_HOSTS");
        if allowed_hosts.is_empty() {
            allowed_hosts = vec!["localhost".into(), "127.0.0.1".into()];
            if let Some(host) = public_url.split("://").nth(1).and_then(|rest| rest.split(['/', ':']).next())
            {
                allowed_hosts.push(host.to_owned());
            }
        }
        Ok(Self {
            bind,
            public_url: public_url.trim_end_matches('/').to_owned(),
            database_url: var("GRADO_DATABASE_URL", "sqlite://grado.db"),
            content_dir: var("GRADO_CONTENT_DIR", "content").into(),
            web_dir: var("GRADO_WEB_DIR", "web").into(),
            auth_mode,
            admin_emails: list("GRADO_ADMIN_EMAILS").into_iter().map(|e| e.to_lowercase()).collect(),
            stamp_ttl_hours: var("GRADO_STAMP_TTL_HOURS", "24").parse()?,
            session_ttl_days: var("GRADO_SESSION_TTL_DAYS", "30").parse()?,
            allowed_hosts,
            grader: GraderConfig {
                anthropic_api_key: env::var("ANTHROPIC_API_KEY").ok().filter(|k| !k.is_empty()),
                model: var("GRADO_GRADER_MODEL", "claude-opus-5-5"),
                effort: var("GRADO_GRADER_EFFORT", "medium"),
            },
        })
    }

    pub fn is_admin(&self, email: &str) -> bool {
        self.admin_emails.iter().any(|a| a == &email.to_lowercase())
    }
}

fn var(name: &str, default: &str) -> String {
    env::var(name).ok().filter(|v| !v.is_empty()).unwrap_or_else(|| default.to_owned())
}

fn list(name: &str) -> Vec<String> {
    env::var(name)
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect()
}
