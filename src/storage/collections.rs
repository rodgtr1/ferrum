use anyhow::Result;
use std::path::PathBuf;
use crate::models::collection::Collection;

fn collections_dir() -> Result<PathBuf> {
    let base = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot find config directory"))?;
    let dir = base.join("ferrum").join("collections");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn load_all() -> Result<Vec<Collection>> {
    let dir = collections_dir()?;
    let mut collections = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            let content = std::fs::read_to_string(&path)?;
            match serde_json::from_str::<Collection>(&content) {
                Ok(c) => collections.push(c),
                Err(e) => eprintln!("Failed to parse collection {:?}: {}", path, e),
            }
        }
    }
    collections.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(collections)
}

pub fn save(collection: &Collection) -> Result<()> {
    let dir = collections_dir()?;
    let path = dir.join(format!("{}.json", collection.id));
    let content = serde_json::to_string_pretty(collection)?;
    std::fs::write(path, content)?;
    Ok(())
}

pub fn delete(id: &str) -> Result<()> {
    let dir = collections_dir()?;
    let path = dir.join(format!("{}.json", id));
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}
