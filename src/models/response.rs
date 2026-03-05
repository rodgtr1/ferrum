use serde::{Deserialize, Serialize};
use super::request::KeyValuePair;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseData {
    pub status_code: u16,
    pub status_text: String,
    pub headers: Vec<KeyValuePair>,
    pub body: String,
    pub elapsed_ms: u128,
    pub size_bytes: usize,
}

impl ResponseData {
    pub fn status_display(&self) -> String {
        format!("{} {}", self.status_code, self.status_text)
    }

    pub fn size_display(&self) -> String {
        if self.size_bytes < 1024 {
            format!("{} B", self.size_bytes)
        } else if self.size_bytes < 1024 * 1024 {
            format!("{:.1} KB", self.size_bytes as f64 / 1024.0)
        } else {
            format!("{:.1} MB", self.size_bytes as f64 / (1024.0 * 1024.0))
        }
    }
}
