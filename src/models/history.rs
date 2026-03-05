use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::request::{HttpMethod, KeyValuePair};
use super::response::ResponseData;

/// A sanitized snapshot of a request safe to persist to disk.
/// Auth credentials are intentionally excluded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryRequest {
    pub url: String,
    pub method: HttpMethod,
    pub headers: Vec<KeyValuePair>,
    pub query_params: Vec<KeyValuePair>,
    // body omitted — can be large and may contain sensitive data
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub request: HistoryRequest,
    pub status_code: Option<u16>,
    pub elapsed_ms: Option<u128>,
    pub error: Option<String>,
}

impl HistoryEntry {
    pub fn from_request(url: &str, method: &HttpMethod, headers: &[KeyValuePair], params: &[KeyValuePair]) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            request: HistoryRequest {
                url: url.to_string(),
                method: method.clone(),
                headers: headers.to_vec(),
                query_params: params.to_vec(),
            },
            status_code: None,
            elapsed_ms: None,
            error: None,
        }
    }

    pub fn with_response(mut self, response: &ResponseData) -> Self {
        self.status_code = Some(response.status_code);
        self.elapsed_ms = Some(response.elapsed_ms);
        self
    }

    pub fn with_error(mut self, error: String) -> Self {
        self.error = Some(error);
        self
    }
}

pub const MAX_HISTORY: usize = 500;
