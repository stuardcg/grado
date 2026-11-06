//! One use case per file (ADR 0004 rules 5 and 6). Each takes the ports it needs as `&dyn` and
//! its `Input`, and returns `Result<Output, Error>`.

pub mod authenticate_agent_connection;
pub mod get_briefing;
pub mod get_criteria;
pub mod list_my_projects;
