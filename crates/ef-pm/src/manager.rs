use std::path::PathBuf;
use anyhow::Result;

pub struct PluginManager {
    base_dir: PathBuf,
}

impl PluginManager {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    pub fn init(&self) -> Result<()> {
        std::fs::create_dir_all(self.base_dir.join("plugins"))?;
        Ok(())
    }

    pub fn install(&self, url: &str) -> Result<PathBuf> {
        let name = url.split('/').last().unwrap_or("plugin");
        let dest = self.base_dir.join("plugins").join(name);
        if !dest.exists() {
            git2::Repository::clone(url, &dest)?;
        }
        Ok(dest)
    }

    pub fn list(&self) -> Result<Vec<String>> {
        let plugins_dir = self.base_dir.join("plugins");
        if !plugins_dir.exists() {
            return Ok(vec![]);
        }
        Ok(std::fs::read_dir(plugins_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect())
    }
}
