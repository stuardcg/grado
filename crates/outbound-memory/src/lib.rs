//! In-memory implementations of the ports. Temporary: they run the app until a database adapter
//! replaces them (ADR 0004). Everything is lost when the process stops.

pub mod agent_connections;
pub mod catalog;
pub mod progress;
pub mod sample;
pub mod settings;
pub mod stamp_codes;

use std::sync::{Mutex, MutexGuard};

use application::ports::PortError;

/// A poisoned lock means another request panicked mid-write, so the data can't be trusted.
fn lock<T>(mutex: &Mutex<T>) -> Result<MutexGuard<'_, T>, PortError> {
    mutex.lock().map_err(|error| PortError::unavailable(error.to_string()))
}
