use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HfModelInfo {
    pub id: String,
    #[serde(default)]
    pub siblings: Vec<HfSibling>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HfSibling {
    pub rfilename: String,
    pub size: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HfGgufFile {
    pub repo_id: String,
    pub filename: String,
    pub size_bytes: Option<u64>,
    pub download_url: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LlmfitRuntimeState {
    Stopped,
    Starting,
    Ready,
    Unhealthy,
    Failed,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmfitStatus {
    pub state: LlmfitRuntimeState,
    pub pid: Option<u32>,
    pub base_url: String,
    pub last_error: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FitModelQuery {
    pub search: Option<String>,
    pub fit: String,
    pub include_too_tight: bool,
    pub limit: u16,
    pub sort: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FitModel {
    pub name: String,
    pub provider: Option<String>,
    pub params_b: Option<f64>,
    pub context_length: Option<u32>,
    pub use_case: Option<String>,
    pub fit_level: Option<String>,
    pub fit_label: Option<String>,
    pub run_mode_label: Option<String>,
    pub score: Option<f64>,
    pub estimated_tps: Option<f64>,
    pub runtime: Option<String>,
    pub runtime_label: Option<String>,
    pub best_quant: Option<String>,
    pub memory_required_gb: Option<f64>,
    pub memory_available_gb: Option<f64>,
    pub utilization_pct: Option<f64>,
    pub gguf_sources: Vec<String>,
}

impl FitModel {
    pub fn from_value(value: &serde_json::Value) -> Option<Self> {
        let name = value.get("name")?.as_str()?.to_string();
        let gguf_sources = value
            .get("gguf_sources")
            .and_then(|sources| sources.as_array())
            .map(|sources| {
                sources
                    .iter()
                    .filter_map(|source| source.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();

        Some(Self {
            name,
            provider: string_field(value, "provider"),
            params_b: value.get("params_b").and_then(|field| field.as_f64()),
            context_length: value
                .get("context_length")
                .and_then(|field| field.as_u64())
                .and_then(|field| u32::try_from(field).ok()),
            use_case: string_field(value, "use_case"),
            fit_level: string_field(value, "fit_level"),
            fit_label: string_field(value, "fit_label"),
            run_mode_label: string_field(value, "run_mode_label"),
            score: value.get("score").and_then(|field| field.as_f64()),
            estimated_tps: value.get("estimated_tps").and_then(|field| field.as_f64()),
            runtime: string_field(value, "runtime"),
            runtime_label: string_field(value, "runtime_label"),
            best_quant: string_field(value, "best_quant"),
            memory_required_gb: value
                .get("memory_required_gb")
                .and_then(|field| field.as_f64()),
            memory_available_gb: value
                .get("memory_available_gb")
                .and_then(|field| field.as_f64()),
            utilization_pct: value
                .get("utilization_pct")
                .and_then(|field| field.as_f64()),
            gguf_sources,
        })
    }
}

fn string_field(value: &serde_json::Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(|field| field.as_str())
        .map(str::to_string)
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DownloadStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelDownload {
    pub id: u64,
    pub repo_id: String,
    pub filename: String,
    pub destination: String,
    pub received_bytes: u64,
    pub total_bytes: Option<u64>,
    pub status: DownloadStatus,
    pub error: Option<String>,
}
