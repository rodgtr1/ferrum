use anyhow::Result;
use std::path::PathBuf;
use crate::models::history::{HistoryEntry, MAX_HISTORY};

fn history_path() -> Result<PathBuf> {
    let base = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot find config directory"))?;
    let dir = base.join("ferrum");
    std::fs::create_dir_all(&dir)?;
    Ok(dir.join("history.json"))
}

pub fn load() -> Result<Vec<HistoryEntry>> {
    let path = history_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(&path)?;
    let entries: Vec<HistoryEntry> = serde_json::from_str(&content).unwrap_or_default();
    Ok(entries)
}

pub fn save(entries: &[HistoryEntry]) -> Result<()> {
    let path = history_path()?;
    let truncated = if entries.len() > MAX_HISTORY {
        &entries[entries.len() - MAX_HISTORY..]
    } else {
        entries
    };
    let content = serde_json::to_string_pretty(truncated)?;
    std::fs::write(path, content)?;
    Ok(())
}
