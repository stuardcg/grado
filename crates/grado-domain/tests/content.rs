//! Loads the real `content/` directory, so a broken level fails CI, not a session.

use std::path::PathBuf;
use std::process::Command;

use grado_domain::{Catalog, Check};

fn content_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../content")
}

#[test]
fn content_loads_and_levels_are_in_order() {
    let catalog = Catalog::load(content_root()).expect("content loads");
    let ids: Vec<&str> = catalog.levels().iter().map(|l| l.id.as_str()).collect();
    assert_eq!(ids.first(), Some(&"00-speedrun"));
    assert_eq!(ids.last(), Some(&"08-finish-line"));
    for level in catalog.levels() {
        assert!(!level.brief_md.trim().is_empty(), "{} has an empty brief", level.id);
        assert!(!level.page_md.trim().is_empty(), "{} has an empty page", level.id);
        for criterion in &level.spec.criteria {
            assert!(
                !criterion.ask.contains("{{"),
                "{} {} has an unrendered placeholder",
                level.id,
                criterion.id
            );
        }
    }
    assert!(catalog.shared_file("registrations.csv").is_some());
}

/// The hidden expected values must agree with the reference implementation
/// in `tools/reference_answers.py`. Skipped when python3 is unavailable.
#[test]
fn expected_values_match_reference() {
    let Ok(output) =
        Command::new("python3").arg(content_root().join("../tools/reference_answers.py")).output()
    else {
        eprintln!("python3 not found; skipping");
        return;
    };
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let answers = String::from_utf8(output.stdout).unwrap();
    let catalog = Catalog::load(content_root()).unwrap();

    let expect = |level: &str, criterion: &str, section: &str| {
        let check = &catalog.level(level).unwrap().criterion(criterion).unwrap().check;
        let Check::KeyValues { expected, .. } = check else {
            panic!("{level}/{criterion} is not key_values")
        };
        let block = answers.split(section).nth(1).unwrap().split('}').next().unwrap();
        for (key, value) in expected {
            let line = format!("\"{key}\": {value}");
            assert!(block.contains(&line), "{level}/{criterion}: reference has no `{line}`");
        }
    };
    expect("02-learning", "count-script", "riders_small.csv");
    expect("05-prototype", "summary", "registrations.csv (levels 5-7)");
    expect("07-refactoring", "same-summary", "registrations.csv (levels 5-7)");
    expect("08-finish-line", "capacity-summary", "with Elite capacity");
}
