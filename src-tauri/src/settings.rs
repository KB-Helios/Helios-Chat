use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HeliosSettings {
    pub default_model_id: Option<String>,
    pub system_prompt: String,
    pub temperature: f32,
    pub top_p: f32,
    pub max_tokens: u32,
    pub n_ctx: u32,
    pub kv_type_k: String,
    pub kv_type_v: String,
    pub n_gpu_layers: i32,
    pub idle_unload_minutes: u32,
    pub engine_port: u16,
}

impl Default for HeliosSettings {
    fn default() -> Self {
        Self {
            default_model_id: None,
            system_prompt: "You are Helios, a local AI assistant running through EIE.".to_string(),
            temperature: 0.7,
            top_p: 0.9,
            max_tokens: 1024,
            n_ctx: 4096,
            kv_type_k: "turbo3".to_string(),
            kv_type_v: "turbo3".to_string(),
            n_gpu_layers: 99,
            idle_unload_minutes: 20,
            engine_port: 8090,
        }
    }
}

pub fn load_settings(path: &Path) -> anyhow::Result<HeliosSettings> {
    if !path.exists() {
        return Ok(HeliosSettings::default());
    }

    let raw = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&raw)?)
}

pub fn save_settings(path: &Path, settings: &HeliosSettings) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let raw = serde_json::to_string_pretty(settings)?;
    std::fs::write(path, raw)?;
    Ok(())
}
