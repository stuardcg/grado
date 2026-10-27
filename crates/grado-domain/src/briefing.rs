//! Assembles what the coaching agent reads at the start of a session:
//! the shared coach, the current level's briefing, and the player's state.

use std::fmt::Write as _;

use crate::catalog::Catalog;
use crate::level::Level;

/// Player state the briefing needs, gathered by the caller from storage.
#[derive(Debug, Default)]
pub struct BriefingState {
    pub player_name: String,
    pub previous_handoff: Option<String>,
    pub checks_passed: usize,
    pub guides_unlocked: u8,
    pub parked_ideas: Vec<String>,
}

pub fn render(catalog: &Catalog, level: &Level, state: &BriefingState) -> String {
    let spec = &level.spec;
    let mut out = String::new();
    out.push_str(catalog.core_md.trim_end());
    out.push_str("\n\n---\n\n");

    let _ = writeln!(out, "# Checkpoint {}: {} ({})\n", spec.number, spec.title, spec.environment.label());
    let _ = writeln!(out, "- **Player:** {}", state.player_name);
    let _ = writeln!(out, "- **Habit for this checkpoint:** {}", spec.habit);
    let _ = writeln!(out, "- **Objective:** {}", spec.objective);
    let _ = writeln!(out, "- **She walks away with:** {}", spec.produces);
    let _ = writeln!(
        out,
        "- **Checks passed:** {} of {}. Call `get_current_check` to see the one she is on.\n",
        state.checks_passed,
        spec.criteria.len()
    );

    out.push_str(level.brief_md.trim_end());
    out.push_str("\n\n");

    let _ = writeln!(out, "## Handoff from the previous checkpoint\n");
    match &state.previous_handoff {
        Some(handoff) => out.push_str(handoff.trim()),
        None => out.push_str("None. This is where her journey starts."),
    }
    out.push_str("\n\n");

    if !state.parked_ideas.is_empty() {
        let _ = writeln!(out, "## Ideas she parked for later\n");
        for idea in &state.parked_ideas {
            let _ = writeln!(out, "- {idea}");
        }
        out.push('\n');
    }

    let total_guides = level.guides.len();
    if total_guides > 0 {
        let _ = writeln!(
            out,
            "## Help\n\nShe has opened {} of {} guides for this checkpoint. Use `get_guide` when she is stuck; it opens the next one.",
            state.guides_unlocked, total_guides
        );
    }
    out
}
