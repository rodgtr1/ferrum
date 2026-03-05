use anyhow::Result;
use std::path::PathBuf;
use crate::models::environment::Environment;

fn envs_dir() -> Result<PathBuf> {
    let base = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot find config directory"))?;
    let dir = base.join("ferrum").join("environments");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn load_all() -> Result<Vec<Environment>> {
    let dir = envs_dir()?;
    let mut envs = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            let content = std::fs::read_to_string(&path)?;
            match serde_json::from_str::<Environment>(&content) {
                Ok(e) => envs.push(e),
                Err(err) => eprintln!("Failed to parse environment {:?}: {}", path, err),
            }
        }
    }
    envs.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(envs)
}

/// Save environment using its UUID as the filename to prevent path traversal.
pub fn save(env: &Environment) -> Result<()> {
    let dir = envs_dir()?;
    let path = dir.join(format!("{}.json", env.id));
    let content = serde_json::to_string_pretty(env)?;
    std::fs::write(path, content)?;
    Ok(())
}
