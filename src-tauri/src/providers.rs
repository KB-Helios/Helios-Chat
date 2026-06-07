use crate::eie::{ChatRequest, ChatResponse};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ChatProvider {
    pub id: String,
    pub kind: String,
    pub label: String,
    pub enabled: bool,
    pub requires_key: bool,
    pub has_key: bool,
    pub base_url: Option<String>,
    pub models: Vec<String>,
}

pub fn list_providers(key_path: &Path) -> anyhow::Result<Vec<ChatProvider>> {
    let keys = read_keys(key_path)?;
    Ok(provider_definitions()
        .into_iter()
        .map(|mut provider| {
            provider.has_key = keys.contains_key(&provider.id);
            provider.enabled = !provider.requires_key || provider.has_key;
            provider
        })
        .collect())
}

pub fn provider_key_exists(key_path: &Path, provider_id: &str) -> anyhow::Result<bool> {
    Ok(read_keys(key_path)?.contains_key(provider_id))
}

pub fn set_provider_key(key_path: &Path, provider_id: &str, key: &str) -> anyhow::Result<()> {
    let trimmed = key.trim();
    if trimmed.is_empty() {
        anyhow::bail!("API key cannot be empty");
    }
    let mut keys = read_keys(key_path)?;
    keys.insert(provider_id.to_string(), trimmed.to_string());
    write_keys(key_path, &keys)
}

pub fn delete_provider_key(key_path: &Path, provider_id: &str) -> anyhow::Result<()> {
    let mut keys = read_keys(key_path)?;
    keys.remove(provider_id);
    write_keys(key_path, &keys)
}

pub async fn send_cloud_chat_request(
    key_path: &Path,
    request: &ChatRequest,
) -> anyhow::Result<ChatResponse> {
    let provider_id = request.provider_id.as_deref().unwrap_or("eie-local");
    if provider_id == "eie-local" {
        anyhow::bail!("EIE requests are handled by the local engine");
    }

    let key = read_keys(key_path)?
        .remove(provider_id)
        .ok_or_else(|| anyhow::anyhow!("{} API key is not configured", provider_id))?;

    match provider_id {
        "openai" => send_openai_compatible(
            "https://api.openai.com/v1/chat/completions",
            &key,
            request,
        )
        .await,
        "openai-compatible" => {
            let base_url = request
                .base_url
                .as_deref()
                .unwrap_or("http://127.0.0.1:1234/v1");
            let url = openai_compatible_chat_url(base_url);
            send_openai_compatible(&url, &key, request).await
        }
        "anthropic" => send_anthropic(&key, request).await,
        "google" => send_google(&key, request).await,
        _ => anyhow::bail!("Unsupported provider: {}", provider_id),
    }
}

pub fn openai_compatible_chat_url(base_url: &str) -> String {
    let base = base_url.trim_end_matches('/');
    let base = base.strip_suffix("/v1").unwrap_or(base);
    format!("{}/v1/chat/completions", base)
}

pub fn provider_http_error_message(provider: &str, status: impl std::fmt::Display, body: &str) -> String {
    let body = body.trim();
    if body.is_empty() {
        format!("{} API error ({})", provider, status)
    } else {
        format!("{} API error ({}): {}", provider, status, body)
    }
}

fn provider_definitions() -> Vec<ChatProvider> {
    vec![
        ChatProvider {
            id: "eie-local".to_string(),
            kind: "eie-local".to_string(),
            label: "EIE Local".to_string(),
            enabled: true,
            requires_key: false,
            has_key: false,
            base_url: Some("http://127.0.0.1:8090/v1".to_string()),
            models: vec!["qwen3-4b-q4-k-m".to_string(), "qwen3-8b-q4-k-m".to_string()],
        },
        ChatProvider {
            id: "openai".to_string(),
            kind: "openai".to_string(),
            label: "OpenAI".to_string(),
            enabled: false,
            requires_key: true,
            has_key: false,
            base_url: Some("https://api.openai.com/v1".to_string()),
            models: vec![
                "gpt-4.1".to_string(),
                "gpt-4.1-mini".to_string(),
                "gpt-4o".to_string(),
            ],
        },
        ChatProvider {
            id: "anthropic".to_string(),
            kind: "anthropic".to_string(),
            label: "Anthropic".to_string(),
            enabled: false,
            requires_key: true,
            has_key: false,
            base_url: Some("https://api.anthropic.com".to_string()),
            models: vec![
                "claude-4-sonnet".to_string(),
                "claude-4-opus".to_string(),
                "claude-3-5-haiku-latest".to_string(),
            ],
        },
        ChatProvider {
            id: "google".to_string(),
            kind: "google".to_string(),
            label: "Google Gemini".to_string(),
            enabled: false,
            requires_key: true,
            has_key: false,
            base_url: Some("https://generativelanguage.googleapis.com".to_string()),
            models: vec![
                "gemini-2.5-pro".to_string(),
                "gemini-2.5-flash".to_string(),
                "gemini-2.0-flash".to_string(),
            ],
        },
        ChatProvider {
            id: "openai-compatible".to_string(),
            kind: "openai-compatible".to_string(),
            label: "OpenAI-compatible".to_string(),
            enabled: false,
            requires_key: true,
            has_key: false,
            base_url: Some("http://127.0.0.1:1234/v1".to_string()),
            models: vec!["local-model".to_string()],
        },
    ]
}

fn read_keys(path: &Path) -> anyhow::Result<BTreeMap<String, String>> {
    if !path.exists() {
        return Ok(BTreeMap::new());
    }
    let raw = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&raw)?)
}

fn write_keys(path: &Path, keys: &BTreeMap<String, String>) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_string_pretty(keys)?)?;
    Ok(())
}

async fn send_openai_compatible(
    url: &str,
    key: &str,
    request: &ChatRequest,
) -> anyhow::Result<ChatResponse> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()?;
    let response = client
        .post(url)
        .bearer_auth(key)
        .json(&serde_json::json!({
            "model": request.model,
            "messages": request.messages,
            "temperature": request.temperature,
            "top_p": request.top_p,
            "max_tokens": request.max_tokens,
            "stream": false
        }))
        .send()
        .await?;
    if !response.status().is_success() {
        let status = response.status();
        let error_body = response.text().await.unwrap_or_default();
        anyhow::bail!("{}", provider_http_error_message("OpenAI", status, &error_body));
    }
    let value = response.json::<serde_json::Value>().await?;
    let content = value
        .pointer("/choices/0/message/content")
        .and_then(|content| content.as_str())
        .unwrap_or_default()
        .to_string();
    if content.is_empty() {
        anyhow::bail!("Provider returned an empty response");
    }
    Ok(ChatResponse {
        conversation_id: request
            .conversation_id
            .clone()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        content,
        citations: Vec::new(),
    })
}

async fn send_anthropic(key: &str, request: &ChatRequest) -> anyhow::Result<ChatResponse> {
    let system = request
        .messages
        .iter()
        .find(|message| message.role == "system")
        .map(|message| message.content.clone());
    let messages: Vec<_> = request
        .messages
        .iter()
        .filter(|message| message.role == "user" || message.role == "assistant")
        .map(|message| serde_json::json!({ "role": message.role, "content": message.content }))
        .collect();

    let mut body = serde_json::json!({
        "model": request.model,
        "messages": messages,
        "temperature": request.temperature,
        "top_p": request.top_p,
        "max_tokens": request.max_tokens
    });
    if let Some(system) = system {
        body["system"] = serde_json::Value::String(system);
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()?;
    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", key)
        .header("anthropic-version", "2023-06-01")
        .json(&body)
        .send()
        .await?;
    if !response.status().is_success() {
        let status = response.status();
        let error_body = response.text().await.unwrap_or_default();
        anyhow::bail!("{}", provider_http_error_message("Anthropic", status, &error_body));
    }
    let value = response.json::<serde_json::Value>().await?;
    let content = value
        .pointer("/content/0/text")
        .and_then(|content| content.as_str())
        .unwrap_or_default()
        .to_string();
    if content.is_empty() {
        anyhow::bail!("Anthropic returned an empty response");
    }
    Ok(ChatResponse {
        conversation_id: request
            .conversation_id
            .clone()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        content,
        citations: Vec::new(),
    })
}

async fn send_google(key: &str, request: &ChatRequest) -> anyhow::Result<ChatResponse> {
    let contents: Vec<_> = request
        .messages
        .iter()
        .filter(|message| message.role != "system")
        .map(|message| {
            let role = if message.role == "assistant" { "model" } else { "user" };
            serde_json::json!({
                "role": role,
                "parts": [{ "text": message.content }]
            })
        })
        .collect();
    let system_instruction = request
        .messages
        .iter()
        .find(|message| message.role == "system")
        .map(|message| serde_json::json!({ "parts": [{ "text": message.content }] }));
    let mut body = serde_json::json!({
        "contents": contents,
        "generationConfig": {
            "temperature": request.temperature,
            "topP": request.top_p,
            "maxOutputTokens": request.max_tokens
        }
    });
    if let Some(system_instruction) = system_instruction {
        body["systemInstruction"] = system_instruction;
    }
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        request.model, key
    );
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()?;
    let response = client
        .post(url)
        .json(&body)
        .send()
        .await?;
    if !response.status().is_success() {
        let status = response.status();
        let error_body = response.text().await.unwrap_or_default();
        anyhow::bail!("{}", provider_http_error_message("Google", status, &error_body));
    }
    let value = response.json::<serde_json::Value>().await?;
    let content = value
        .pointer("/candidates/0/content/parts/0/text")
        .and_then(|content| content.as_str())
        .unwrap_or_default()
        .to_string();
    if content.is_empty() {
        anyhow::bail!("Google returned an empty response");
    }
    Ok(ChatResponse {
        conversation_id: request
            .conversation_id
            .clone()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        content,
        citations: Vec::new(),
    })
}
