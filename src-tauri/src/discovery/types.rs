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

#[derive(Clone, Debug, Deserialize, Serialize)]
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
