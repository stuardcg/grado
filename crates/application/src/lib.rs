//! Use cases and ports, and `Grado`, the application handle inbound adapters call them through
//! (ADR 0010).

pub mod ports;
pub mod use_cases;

#[cfg(any(test, feature = "testkit"))]
pub mod testkit;

use std::sync::Arc;

pub use ports::PortError;
use ports::agent_connections::AgentConnections;
pub use ports::agent_connections::AgentToken;
use ports::catalog::Catalog;
use ports::progress::ProgressStore;
use ports::settings::SettingsStore;
use ports::stamp_codes::StampCodes;
use use_cases::{
    authenticate_agent_connection, get_briefing, get_criteria, issue_stamp, list_my_projects, submit_report,
};

/// The use cases, with the ports they run on. The composition root builds one; inbound adapters
/// call its methods. The ports are private, so nothing outside `application` can call one
/// (ADR 0004 rule 14).
pub struct Grado {
    catalog: Arc<dyn Catalog>,
    progress: Arc<dyn ProgressStore>,
    settings: Arc<dyn SettingsStore>,
    connections: Arc<dyn AgentConnections>,
    stamp_codes: Arc<dyn StampCodes>,
}

impl Grado {
    pub fn new(
        catalog: Arc<dyn Catalog>,
        progress: Arc<dyn ProgressStore>,
        settings: Arc<dyn SettingsStore>,
        connections: Arc<dyn AgentConnections>,
        stamp_codes: Arc<dyn StampCodes>,
    ) -> Self {
        Self { catalog, progress, settings, connections, stamp_codes }
    }

    pub async fn authenticate_agent_connection(
        &self,
        input: authenticate_agent_connection::Input,
    ) -> Result<authenticate_agent_connection::Output, authenticate_agent_connection::Error> {
        authenticate_agent_connection::run(&*self.connections, input).await
    }

    pub async fn list_my_projects(
        &self,
        input: list_my_projects::Input,
    ) -> Result<list_my_projects::Output, list_my_projects::Error> {
        list_my_projects::run(&*self.catalog, &*self.progress, input).await
    }

    pub async fn get_briefing(
        &self,
        input: get_briefing::Input,
    ) -> Result<get_briefing::Output, get_briefing::Error> {
        get_briefing::run(&*self.catalog, &*self.progress, input).await
    }

    pub async fn get_criteria(
        &self,
        input: get_criteria::Input,
    ) -> Result<get_criteria::Output, get_criteria::Error> {
        get_criteria::run(&*self.catalog, &*self.progress, input).await
    }

    pub async fn submit_report(
        &self,
        input: submit_report::Input,
    ) -> Result<submit_report::Output, submit_report::Error> {
        submit_report::run(&*self.catalog, &*self.progress, &*self.settings, input).await
    }

    pub async fn issue_stamp(
        &self,
        input: issue_stamp::Input,
    ) -> Result<issue_stamp::Output, issue_stamp::Error> {
        issue_stamp::run(&*self.catalog, &*self.progress, &*self.settings, &*self.stamp_codes, input).await
    }
}
