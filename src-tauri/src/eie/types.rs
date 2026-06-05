use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum EieBinarySource {
    UserPath,
    BundledSidecar,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ConfigPreset {
    Generic,
    Development,
    Custom,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EieSettings {
    pub binary_source: EieBinarySource,
    pub binary_path: Option<PathBuf>,
    pub model_directory: Option<PathBuf>,
    pub host: String,
    pub port: u16,
    pub context_length: u32,
    pub gpu_layers: u16,
    pub config_preset: ConfigPreset,
    pub auto_start: bool,
    #[serde(default)]
    pub llmfit_binary_path: Option<PathBuf>,
    #[serde(default = "default_llmfit_port")]
    pub llmfit_port: u16,
    #[serde(default)]
    pub auto_start_llmfit: bool,
}

fn default_llmfit_port() -> u16 {
    8787
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EieError {
    pub code: String,
    pub message: String,
}

impl EieError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

pub type EieResult<T> = Result<T, EieError>;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum EieRuntimeState {
    Stopped,
    Starting,
    Ready,
    Unhealthy,
    Stopping,
    Failed,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EieStatus {
    pub state: EieRuntimeState,
    pub pid: Option<u32>,
    pub base_url: String,
    pub config_path: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EieLogLine {
    pub stream: String,
    pub line: String,
    pub timestamp: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EieConfigPreview {
    pub path: String,
    pub yaml: String,
}
