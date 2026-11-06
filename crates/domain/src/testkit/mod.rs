//! Code shared by this crate's tests, compiled only for them.
//!
//! - `builders`: turn short strings into ids, text, steps and reports, so a test reads as data.
//! - `fixtures`: example data made with the builders.
//!
//! Add to the file whose kind fits. Something that fits neither is a new kind: ask before adding
//! a file for it, and name the file for what it holds (never `helpers` or `utils`, rule 20).

pub mod builders;
pub mod fixtures;
