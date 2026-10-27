use std::sync::Arc;

use crate::config::Config;
use crate::trail::Trail;

#[derive(Clone)]
pub struct AppState {
    pub trail: Arc<Trail>,
    pub config: Arc<Config>,
}
