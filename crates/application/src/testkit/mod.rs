//! Each port's shared suite and its in-memory fake, plus the fixtures they use (ADR 0004 "Tests").
//! Compiled only for tests: `application`'s own, and adapters' through the `testkit` feature.

pub mod agent_connections;
pub mod catalog;
pub mod fixtures;
pub mod progress;
pub mod settings;
pub mod stamp_codes;
