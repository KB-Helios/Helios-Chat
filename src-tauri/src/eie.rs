use crate::settings::HeliosSettings;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EieConfigInput {
    pub host: String,
    pub port: u16,
    pub model_dir: PathBuf,
    pub default_model_alias: Option<String>,
    pub default_model_path: Option<PathBuf>,
    pub n_ctx: u32,
    pub type_k: String,
    pub type_v: String,
    pub n_gpu_layers: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EngineStatus {
    pub running: bool,
    pub endpoint: String,
    pub pid: Option<u32>,
    pub detail: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildResult {
    pub backend: String,
    pub binary_path: PathBuf,
    pub log_path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatRequest {
    pub conversation_id: Option<String>,
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: f32,
    pub top_p: f32,
    pub max_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatResponse {
    pub conversation_id: String,
    pub content: String,
}

#[derive(Debug, Default)]
pub struct EngineRuntime {
    child: Option<Child>,
    endpoint: Option<String>,
    pid: Option<u32>,
}

impl EngineRuntime {
    pub fn status(&self, default_port: u16) -> EngineStatus {
        let endpoint = self
            .endpoint
            .clone()
            .unwrap_or_else(|| format!("http://127.0.0.1:{}", default_port));
        EngineStatus {
            running: self.child.is_some(),
            endpoint,
            pid: self.pid,
            detail: if self.child.is_some() {
                "EIE process is managed by Helios.".to_string()
            } else {
                "EIE is not running.".to_string()
            },
        }
    }

    pub fn stop(&mut self) -> anyhow::Result<()> {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
        }
        self.pid = None;
        self.endpoint = None;
        Ok(())
    }

    pub fn start(
        &mut self,
        binary_path: &Path,
        config_path: &Path,
        port: u16,
    ) -> anyhow::Result<EngineStatus> {
        self.stop()?;
        let child = Command::new(binary_path)
            .arg("--config")
            .arg(config_path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        let pid = child.id();
        let endpoint = format!("http://127.0.0.1:{}", port);
        self.pid = Some(pid);
        self.endpoint = Some(endpoint.clone());
        self.child = Some(child);
        Ok(EngineStatus {
            running: true,
            endpoint,
            pid: Some(pid),
            detail: "EIE process started.".to_string(),
        })
    }
}

pub fn render_eie_config(input: &EieConfigInput) -> String {
    let model_dir = normalize_path(&input.model_dir);
    let mut yaml = format!(
        "host: {host}\nport: {port}\nstrategy: generic\nmodel_dir: {model_dir}\nauto_discover: true\ntype_k: {type_k}\ntype_v: {type_v}\nflash_attn: true\nn_ctx: {n_ctx}\nn_gpu_layers: {n_gpu_layers}\nreserve_mb: 512\nlog_level: info\n",
        host = input.host,
        port = input.port,
        model_dir = model_dir,
        type_k = input.type_k,
        type_v = input.type_v,
        n_ctx = input.n_ctx,
        n_gpu_layers = input.n_gpu_layers,
    );

    if let (Some(alias), Some(path)) = (&input.default_model_alias, &input.default_model_path) {
        yaml.push_str("models:\n");
        yaml.push_str(&format!("  {}: {}\n", alias, normalize_path(path)));
        yaml.push_str("warm_load:\n");
        yaml.push_str(&format!("  default_model: {}\n", alias));
        yaml.push_str("  idle_unload: smart\n");
    }

    yaml
}

pub fn write_eie_config(path: &Path, input: &EieConfigInput) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, render_eie_config(input))?;
    Ok(())
}

pub async fn send_chat_request(
    app: &AppHandle,
    endpoint: &str,
    request: &ChatRequest,
) -> anyhow::Result<ChatResponse> {
    let url = format!("{}/v1/chat/completions", endpoint.trim_end_matches('/'));
    let client = reqwest::Client::new();
    let response = client
        .post(url)
        .json(&serde_json::json!({
            "model": request.model,
            "messages": request.messages,
            "temperature": request.temperature,
            "top_p": request.top_p,
            "max_tokens": request.max_tokens,
            "stream": true
        }))
        .send()
        .await?
        .error_for_status()?;

    let mut content = String::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        let text = String::from_utf8_lossy(&chunk);
        for token in parse_sse_tokens(&text) {
            content.push_str(&token);
            let _ = app.emit("chat:token", &token);
        }
    }

    if content.is_empty() {
        anyhow::bail!("EIE returned an empty response");
    }

    let _ = app.emit("chat:done", &content);
    Ok(ChatResponse {
        conversation_id: request
            .conversation_id
            .clone()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        content,
    })
}

pub fn config_input_from_settings(
    settings: &HeliosSettings,
    model_dir: PathBuf,
    default_model_path: Option<PathBuf>,
) -> EieConfigInput {
    EieConfigInput {
        host: "127.0.0.1".to_string(),
        port: settings.engine_port,
        model_dir,
        default_model_alias: settings.default_model_id.clone(),
        default_model_path,
        n_ctx: settings.n_ctx,
        type_k: settings.kv_type_k.clone(),
        type_v: settings.kv_type_v.clone(),
        n_gpu_layers: settings.n_gpu_layers,
    }
}

fn parse_sse_tokens(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|line| line.strip_prefix("data: "))
        .filter(|line| *line != "[DONE]")
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .filter_map(|value| {
            value
                .pointer("/choices/0/delta/content")
                .and_then(|token| token.as_str())
                .map(ToString::to_string)
        })
        .collect()
}

fn normalize_path(path: &Path) -> String {
    path.display().to_string().replace('\\', "/")
}
