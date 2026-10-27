use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Terrain a level sits on. Difficulty rises with altitude.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Environment {
    Meadow,
    Forest,
    Canyon,
    Ridge,
    Peak,
}

impl Environment {
    pub fn label(self) -> &'static str {
        match self {
            Self::Meadow => "Meadow",
            Self::Forest => "Forest trail",
            Self::Canyon => "Canyon",
            Self::Ridge => "Rocky ridge",
            Self::Peak => "Snowy peak",
        }
    }
}

/// `level.toml`, as authored.
#[derive(Debug, Clone, Deserialize)]
pub struct LevelSpec {
    pub number: u32,
    pub title: String,
    pub environment: Environment,
    /// The one working-with-AI habit this level teaches.
    pub habit: String,
    /// One sentence the player sees on the level page.
    pub objective: String,
    /// What the player walks away with.
    pub produces: String,
    #[serde(default)]
    pub estimated_minutes: Option<u32>,
    pub criteria: Vec<Criterion>,
}

/// A level loaded from `content/levels/<id>/`.
#[derive(Debug, Clone)]
pub struct Level {
    /// Directory name, e.g. `01-investigation`. Stable across content edits.
    pub id: String,
    pub spec: LevelSpec,
    /// Agent-facing briefing (`brief.md`).
    pub brief_md: String,
    /// Player-facing page (`page.md`).
    pub page_md: String,
    /// Escalating help, ordered by tier (`guides/<tier>-<name>.md`).
    pub guides: Vec<Guide>,
}

impl Level {
    pub fn number(&self) -> u32 {
        self.spec.number
    }

    pub fn criterion(&self, id: &str) -> Option<&Criterion> {
        self.spec.criteria.iter().find(|c| c.id == id)
    }
}

#[derive(Debug, Clone)]
pub struct Guide {
    pub tier: u8,
    pub title: String,
    pub body_md: String,
}

/// One acceptance criterion. The agent only ever sees `title`, `ask` and
/// `evidence`; `check` stays on the server.
#[derive(Debug, Clone, Deserialize)]
pub struct Criterion {
    pub id: String,
    pub title: String,
    /// What the agent must collect and submit, phrased for the agent.
    pub ask: String,
    pub evidence: EvidenceKind,
    pub check: Check,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    /// Prose the player wrote (notes, explanations, plans).
    Text,
    /// Raw terminal output from running something in her project.
    Output,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Check {
    /// Anything non-trivial was submitted.
    Present { min_chars: usize },
    /// Every needle appears in the evidence.
    ContainsAll {
        needles: Vec<String>,
        #[serde(default)]
        case_sensitive: bool,
    },
    /// Evidence contains `key: value` lines matching `expected`, in any order.
    KeyValues {
        expected: BTreeMap<String, String>,
        #[serde(default = "default_true")]
        allow_extra: bool,
    },
    /// Evidence is a pytest run with at least `min_passed` passing and none failing.
    Pytest { min_passed: u32 },
    /// Every inner check must pass.
    All { checks: Vec<Check> },
    /// Graded against a hidden rubric, by a language model when one is configured.
    Rubric {
        rubric: String,
        #[serde(default)]
        min_chars: usize,
        /// Used only when no model is configured: each inner list is a group of
        /// alternatives, and at least one word from every group must appear.
        #[serde(default)]
        fallback_keywords: Vec<Vec<String>>,
    },
}

fn default_true() -> bool {
    true
}
