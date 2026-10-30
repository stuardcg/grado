//! Players, known to the game rules only by their id.

use shared::id::Id;

/// A person with an account. The rules never need more than its id.
pub enum Player {}

pub type PlayerId = Id<Player>;
