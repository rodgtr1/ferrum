use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::request::RequestConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestItem {
    pub id: String,
    pub name: String,
    #[serde(flatten)]
    pub config: RequestConfig,
}

impl RequestItem {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            config: RequestConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub id: String,
    pub name: String,
    pub requests: Vec<RequestItem>,
    #[serde(default)]
    pub expanded: bool,
}

impl Collection {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            requests: Vec::new(),
            expanded: true,
        }
    }
}
