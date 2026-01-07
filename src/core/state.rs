use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub routes: Arc<HashMap<String, String>>,
}