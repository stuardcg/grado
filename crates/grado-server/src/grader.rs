//! Rubric grading with Claude, over the Messages API.
//!
//! Rust has no official Anthropic SDK, so this is a raw HTTP call. The model
//! returns a JSON verdict constrained by `output_config.format`. Any failure
//! (network, refusal, malformed output) falls back to the offline grader so a
//! player is never blocked by an outage.

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::json;
use grado_domain::{OfflineRubricGrader, RubricGrader, RubricRequest, Verdict};

use crate::config::GraderConfig;

const API_URL: &str = "https://api.anthropic.com/v1/messages";

pub struct ClaudeRubricGrader {
    http: reqwest::Client,
    api_key: String,
    model: String,
    effort: String,
}

impl ClaudeRubricGrader {
    pub fn new(config: &GraderConfig, api_key: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            api_key,
            model: config.model.clone(),
            effort: config.effort.clone(),
        }
    }

    async fn call(&self, request: &RubricRequest<'_>) -> anyhow::Result<ModelVerdict> {
        let system = "You grade one piece of evidence from a beginner learning to build software with an AI coach. \
            Apply the rubric exactly; be generous about wording, strict about substance. \
            The feedback goes to the coach, not the learner: say what is missing or wrong in one or two sentences, \
            without quoting the rubric and without giving away the answer. If it passes, say briefly what was good.";
        let user = format!(
            "<checkpoint>{}: {}</checkpoint>\n<check>{}</check>\n<what_was_asked>{}</what_was_asked>\n<rubric>{}</rubric>\n<evidence>{}</evidence>",
            request.level.spec.title,
            request.level.spec.objective,
            request.criterion.title,
            request.criterion.ask.trim(),
            request.rubric.trim(),
            request.evidence,
        );
        let body = json!({
            "model": self.model,
            "max_tokens": 16000,
            "fallbacks": "default",
            "output_config": {
                "effort": self.effort,
                "format": {
                    "type": "json_schema",
                    "schema": {
                        "type": "object",
                        "properties": {
                            "pass": {"type": "boolean"},
                            "feedback": {"type": "string"}
                        },
                        "required": ["pass", "feedback"],
                        "additionalProperties": false
                    }
                }
            },
            "system": system,
            "messages": [{"role": "user", "content": user}]
        });

        let response = self
            .http
            .post(API_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("anthropic-beta", "server-side-fallback-2026-07-01")
            .json(&body)
            .send()
            .await?;
        let status = response.status();
        let message: serde_json::Value = response.json().await?;
        if !status.is_success() {
            anyhow::bail!("Messages API returned {status}: {message}");
        }
        if message["stop_reason"] == "refusal" {
            anyhow::bail!("model declined to grade");
        }
        let text = message["content"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|block| block["type"] == "text")
            .filter_map(|block| block["text"].as_str())
            .collect::<String>();
        Ok(serde_json::from_str(&text)?)
    }
}

#[derive(Deserialize)]
struct ModelVerdict {
    pass: bool,
    feedback: String,
}

#[async_trait]
impl RubricGrader for ClaudeRubricGrader {
    async fn grade(&self, request: RubricRequest<'_>) -> Verdict {
        match self.call(&request).await {
            Ok(verdict) => {
                Verdict { passed: verdict.pass, feedback: verdict.feedback, grader: self.model.clone() }
            }
            Err(error) => {
                tracing::warn!(%error, "model grading failed; using offline grader");
                let mut verdict = OfflineRubricGrader.grade(request).await;
                verdict.grader = "offline (model unavailable)".into();
                verdict
            }
        }
    }
}
