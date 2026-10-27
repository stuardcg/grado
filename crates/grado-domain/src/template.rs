//! `{{dotted.key}}` substitution for Markdown content.
//!
//! Deliberately tiny: content authors tune values in `coach.toml` without
//! touching prose, and an unknown placeholder fails loading instead of leaking
//! braces into a briefing.

use std::collections::BTreeMap;

use regex::Regex;
use std::sync::LazyLock;

static PLACEHOLDER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{\{\s*([a-zA-Z0-9_.]+)\s*\}\}").expect("valid regex"));

pub type Vars = BTreeMap<String, String>;

/// Flattens a TOML table into `section.key` → string value.
pub fn flatten(table: &toml::Table) -> Vars {
    let mut out = Vars::new();
    flatten_into(&mut out, "", table);
    out
}

fn flatten_into(out: &mut Vars, prefix: &str, table: &toml::Table) {
    for (key, value) in table {
        let path = if prefix.is_empty() { key.clone() } else { format!("{prefix}.{key}") };
        match value {
            toml::Value::Table(inner) => flatten_into(out, &path, inner),
            toml::Value::String(s) => {
                out.insert(path, s.trim_end().to_owned());
            }
            other => {
                out.insert(path, other.to_string());
            }
        }
    }
}

/// Replaces every placeholder, or returns the names that have no value.
pub fn render(source: &str, vars: &Vars) -> Result<String, Vec<String>> {
    let mut missing = Vec::new();
    let rendered = PLACEHOLDER.replace_all(source, |caps: &regex::Captures<'_>| {
        let key = &caps[1];
        match vars.get(key) {
            Some(value) => value.clone(),
            None => {
                missing.push(key.to_owned());
                caps[0].to_owned()
            }
        }
    });
    if missing.is_empty() { Ok(rendered.into_owned()) } else { Err(missing) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_nested_keys_and_reports_missing() {
        let table: toml::Table = toml::from_str("[style]\nmax_lines = 6\n[player]\nname = 'Ana'").unwrap();
        let vars = flatten(&table);
        assert_eq!(render("Hi {{player.name}}, max {{ style.max_lines }}", &vars).unwrap(), "Hi Ana, max 6");
        assert_eq!(render("{{nope}}", &vars).unwrap_err(), vec!["nope".to_string()]);
    }
}
