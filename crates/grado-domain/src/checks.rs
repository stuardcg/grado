//! Judging evidence against a criterion's check.
//!
//! Feedback must help the agent coach without leaking the answer: it names
//! *which* part failed, never the expected value or the rubric text.

use std::collections::BTreeMap;
use std::sync::LazyLock;

use async_trait::async_trait;
use regex::Regex;

use crate::level::{Check, Criterion, Level};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verdict {
    pub passed: bool,
    pub feedback: String,
    /// Which grader produced this verdict, for the audit trail.
    pub grader: String,
}

impl Verdict {
    fn pass(feedback: impl Into<String>, grader: &str) -> Self {
        Self { passed: true, feedback: feedback.into(), grader: grader.to_owned() }
    }

    fn fail(feedback: impl Into<String>, grader: &str) -> Self {
        Self { passed: false, feedback: feedback.into(), grader: grader.to_owned() }
    }
}

/// Everything a rubric grader may look at.
pub struct RubricRequest<'a> {
    pub level: &'a Level,
    pub criterion: &'a Criterion,
    pub rubric: &'a str,
    pub evidence: &'a str,
    pub fallback_keywords: &'a [Vec<String>],
}

#[async_trait]
pub trait RubricGrader: Send + Sync {
    async fn grade(&self, request: RubricRequest<'_>) -> Verdict;
}

/// Grades rubric checks without a model: every keyword group must be matched.
/// Good enough to play the whole journey locally; deploys should use a model.
pub struct OfflineRubricGrader;

#[async_trait]
impl RubricGrader for OfflineRubricGrader {
    async fn grade(&self, request: RubricRequest<'_>) -> Verdict {
        let haystack = request.evidence.to_lowercase();
        let missing = request
            .fallback_keywords
            .iter()
            .filter(|group| !group.iter().any(|word| haystack.contains(&word.to_lowercase())))
            .count();
        if missing == 0 {
            Verdict::pass("Looks complete.", "offline")
        } else {
            Verdict::fail(
                format!(
                    "{missing} part(s) of this check are not covered yet. Re-read the ask and help her fill the gap."
                ),
                "offline",
            )
        }
    }
}

pub async fn evaluate(
    level: &Level,
    criterion: &Criterion,
    evidence: &str,
    rubric_grader: &dyn RubricGrader,
) -> Verdict {
    evaluate_check(level, criterion, &criterion.check, evidence.trim(), rubric_grader).await
}

fn evaluate_check<'a>(
    level: &'a Level,
    criterion: &'a Criterion,
    check: &'a Check,
    evidence: &'a str,
    rubric_grader: &'a dyn RubricGrader,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Verdict> + Send + 'a>> {
    Box::pin(async move {
        match check {
            Check::Present { min_chars } => present(evidence, *min_chars),
            Check::ContainsAll { needles, case_sensitive } => {
                contains_all(evidence, needles, *case_sensitive)
            }
            Check::KeyValues { expected, allow_extra } => key_values(evidence, expected, *allow_extra),
            Check::Pytest { min_passed } => pytest(evidence, *min_passed),
            Check::Rubric { rubric, min_chars, fallback_keywords } => {
                if evidence.chars().count() < *min_chars {
                    return Verdict::fail(
                        "This is too short to judge. She should write it in her own words, a few sentences at least.",
                        "length",
                    );
                }
                rubric_grader
                    .grade(RubricRequest { level, criterion, rubric, evidence, fallback_keywords })
                    .await
            }
            Check::All { checks } => {
                for inner in checks {
                    let verdict = evaluate_check(level, criterion, inner, evidence, rubric_grader).await;
                    if !verdict.passed {
                        return verdict;
                    }
                }
                Verdict::pass("Every part matches.", "all")
            }
        }
    })
}

fn present(evidence: &str, min_chars: usize) -> Verdict {
    if evidence.chars().count() >= min_chars {
        Verdict::pass("Received.", "present")
    } else {
        Verdict::fail("Nothing usable was submitted yet.", "present")
    }
}

fn contains_all(evidence: &str, needles: &[String], case_sensitive: bool) -> Verdict {
    let normalize = |s: &str| {
        if case_sensitive { s.to_owned() } else { s.to_lowercase() }
    };
    let haystack = normalize(evidence);
    let missing = needles.iter().filter(|n| !haystack.contains(&normalize(n))).count();
    if missing == 0 {
        Verdict::pass("Output matches.", "contains")
    } else {
        Verdict::fail(
            format!(
                "The output is missing {missing} expected piece(s). Make sure you pasted the full, unedited output."
            ),
            "contains",
        )
    }
}

/// Parses `key: value` or `key = value` lines. Keys and values are compared
/// case-insensitively with surrounding whitespace ignored.
fn parse_pairs(evidence: &str) -> BTreeMap<String, String> {
    evidence
        .lines()
        .filter_map(|line| {
            let (key, value) = line.split_once(':').or_else(|| line.split_once('='))?;
            Some((normalize_token(key), normalize_token(value)))
        })
        .filter(|(key, _)| !key.is_empty())
        .collect()
}

fn normalize_token(s: &str) -> String {
    s.trim().trim_matches(|c| c == '"' || c == '\'').trim().to_lowercase()
}

fn key_values(evidence: &str, expected: &BTreeMap<String, String>, allow_extra: bool) -> Verdict {
    let actual = parse_pairs(evidence);
    let mut missing = Vec::new();
    let mut wrong = Vec::new();
    for (key, value) in expected {
        match actual.get(&normalize_token(key)) {
            None => missing.push(key.as_str()),
            Some(found) if *found != normalize_token(value) => wrong.push(key.as_str()),
            Some(_) => {}
        }
    }
    let extra: Vec<&str> = if allow_extra {
        Vec::new()
    } else {
        let expected_keys: Vec<String> = expected.keys().map(|k| normalize_token(k)).collect();
        actual.keys().filter(|k| !expected_keys.contains(k)).map(String::as_str).collect()
    };

    if missing.is_empty() && wrong.is_empty() && extra.is_empty() {
        return Verdict::pass("Every value matches.", "key_values");
    }
    let mut parts = Vec::new();
    if !missing.is_empty() {
        parts.push(format!("missing lines for: {}", missing.join(", ")));
    }
    if !wrong.is_empty() {
        parts.push(format!("these values are not right yet: {}", wrong.join(", ")));
    }
    if !extra.is_empty() {
        parts.push(format!("unexpected lines: {}", extra.join(", ")));
    }
    Verdict::fail(
        format!("{}. Help her find why, rather than adjusting numbers by hand.", parts.join("; ")),
        "key_values",
    )
}

static PYTEST_COUNT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(\d+) (passed|failed|errors?)\b").expect("valid regex"));

fn pytest(evidence: &str, min_passed: u32) -> Verdict {
    let mut passed = 0;
    let mut failed = 0;
    for caps in PYTEST_COUNT.captures_iter(evidence) {
        let count: u32 = caps[1].parse().unwrap_or(0);
        match &caps[2] {
            "passed" => passed = count,
            _ => failed += count,
        }
    }
    if passed == 0 && failed == 0 {
        return Verdict::fail(
            "That doesn't look like pytest output. Run `pytest` and paste the whole output, including the summary line.",
            "pytest",
        );
    }
    if failed > 0 {
        return Verdict::fail(format!("{failed} test(s) are failing or erroring."), "pytest");
    }
    if passed < min_passed {
        return Verdict::fail(
            format!("Only {passed} test(s) ran. This check needs at least {min_passed}."),
            "pytest",
        );
    }
    Verdict::pass(format!("{passed} tests passing."), "pytest")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_values_ignores_order_case_and_extra_lines() {
        let expected =
            BTreeMap::from([("Elite".to_string(), "11".to_string()), ("Kids".to_string(), "4".to_string())]);
        let verdict = key_values("Loading...\nkids: 4\nELITE = 11\n", &expected, true);
        assert!(verdict.passed, "{verdict:?}");
    }

    #[test]
    fn key_values_names_wrong_keys_without_revealing_values() {
        let expected =
            BTreeMap::from([("Elite".to_string(), "11".to_string()), ("Kids".to_string(), "4".to_string())]);
        let verdict = key_values("Elite: 12", &expected, true);
        assert!(!verdict.passed);
        assert!(verdict.feedback.contains("Kids"));
        assert!(verdict.feedback.contains("Elite"));
        assert!(!verdict.feedback.contains("11"));
    }

    #[test]
    fn pytest_requires_no_failures_and_enough_passes() {
        assert!(pytest("===== 6 passed in 0.12s =====", 5).passed);
        assert!(!pytest("===== 5 passed, 1 failed in 0.12s =====", 5).passed);
        assert!(!pytest("===== 3 passed in 0.12s =====", 5).passed);
        assert!(!pytest("it works!", 5).passed);
    }

    #[test]
    fn contains_all_is_case_insensitive_by_default() {
        assert!(contains_all("%pdf-1.4 ...", &["%PDF".into()], false).passed);
        assert!(!contains_all("%pdf-1.4 ...", &["%PDF".into()], true).passed);
    }
}
