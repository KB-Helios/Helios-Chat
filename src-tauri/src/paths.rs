use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub root: PathBuf,
    pub models: PathBuf,
    pub downloads: PathBuf,
    pub engine: PathBuf,
    pub logs: PathBuf,
    pub settings: PathBuf,
    pub database: PathBuf,
    pub eie_config: PathBuf,
}

impl AppPaths {
    pub fn resolve(app: &AppHandle) -> anyhow::Result<Self> {
        let root = app.path().app_local_data_dir()?;
        Ok(Self::from_root(root))
    }

    pub fn from_root(root: PathBuf) -> Self {
        Self {
            models: root.join("models"),
            downloads: root.join("downloads"),
            engine: root.join("engine"),
            logs: root.join("logs"),
            settings: root.join("helios.settings.json"),
            database: root.join("helios.sqlite3"),
            eie_config: root.join("eie.engine.yaml"),
            root,
        }
    }

    pub fn ensure(&self) -> anyhow::Result<()> {
        for path in [&self.root, &self.models, &self.downloads, &self.engine, &self.logs] {
            std::fs::create_dir_all(path)?;
        }
        Ok(())
    }
}
