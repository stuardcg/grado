//! Which levels a player may see. Levels unlock strictly in order: the first
//! level not yet completed is the active one, everything after it is locked.

use std::collections::BTreeSet;

use serde::Serialize;

use crate::catalog::Catalog;
use crate::level::Level;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LevelStatus {
    Completed,
    Active,
    Locked,
}

pub struct Journey<'a> {
    catalog: &'a Catalog,
    completed: BTreeSet<String>,
}

impl<'a> Journey<'a> {
    pub fn new(catalog: &'a Catalog, completed: impl IntoIterator<Item = String>) -> Self {
        Self { catalog, completed: completed.into_iter().collect() }
    }

    /// The level the player is working on, or `None` once every level is done.
    pub fn current(&self) -> Option<&'a Level> {
        self.catalog.levels().iter().find(|l| !self.completed.contains(&l.id))
    }

    pub fn status(&self, level_id: &str) -> LevelStatus {
        if self.completed.contains(level_id) {
            LevelStatus::Completed
        } else if self.current().is_some_and(|l| l.id == level_id) {
            LevelStatus::Active
        } else {
            LevelStatus::Locked
        }
    }

    pub fn is_visible(&self, level_id: &str) -> bool {
        self.status(level_id) != LevelStatus::Locked
    }

    pub fn finished(&self) -> bool {
        self.current().is_none()
    }
}
