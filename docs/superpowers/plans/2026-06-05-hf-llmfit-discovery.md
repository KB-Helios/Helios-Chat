# Hugging Face llmfit Discovery Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a Windows-only Discover page that uses a managed local `llmfit serve` helper to rank Hugging Face GGUF model candidates and lets Helios securely download selected GGUF files into the EIE model directory.

**Architecture:** Extend the existing EIE settings object with `llmfit` helper settings so one persisted config drives EIE and discovery. Add a Rust `discovery` module that owns `llmfit` process lifecycle, local `llmfit` REST calls, Hugging Face GGUF metadata, model download jobs, path safety, and progress events. Add React discovery types, Tauri wrappers, hooks, and a `Discover` view that remains UI-only.

**Tech Stack:** Tauri 2, Rust 2021, React 19, TypeScript, shadcn/ui, `@formkit/auto-animate`, Bun tests, `tsgo`, `reqwest` blocking client with rustls for HTTPS metadata/downloads.

---

## File Map

- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/app_config.rs`
- Modify: `src-tauri/src/eie/types.rs`
- Modify: `src-tauri/src/eie/config.rs`
- Create: `src-tauri/src/discovery/mod.rs`
- Create: `src-tauri/src/discovery/types.rs`
- Create: `src-tauri/src/discovery/llmfit.rs`
- Create: `src-tauri/src/discovery/hf.rs`
- Create: `src-tauri/src/discovery/download.rs`
- Create: `src-tauri/src/discovery/manager.rs`
- Create: `src-tauri/src/discovery/commands.rs`
- Modify: `src/lib/eie/types.ts`
- Create: `src/lib/discovery/types.ts`
- Create: `src/lib/discovery/filters.ts`
- Create: `src/lib/discovery/filters.test.ts`
- Create: `src/lib/discovery/downloads.ts`
- Modify: `src/lib/tauri/commands.ts`
- Modify: `src/lib/tauri/events.ts`
- Create: `src/hooks/use-model-discovery.ts`
- Create: `src/app/discover/discover-view.tsx`
- Create: `src/components/discover/discovery-toolbar.tsx`
- Create: `src/components/discover/fit-badge.tsx`
- Create: `src/components/discover/model-fit-table.tsx`
- Create: `src/components/discover/model-detail-panel.tsx`
- Create: `src/components/discover/download-progress-list.tsx`
- Modify: `src/App.tsx`
- Modify: `src/components/app-sidebar.tsx`
- Modify: `src/components/eie/eie-settings-form.tsx`
- Modify: `src/app/settings/settings-view.tsx`

## Tasks

### Task 1: Persist llmfit Settings

**Files:**
- Modify: `src-tauri/src/eie/types.rs`
- Modify: `src-tauri/src/eie/config.rs`
- Modify: `src/lib/eie/types.ts`
- Modify: `src/App.tsx`
- Modify: `src/components/eie/eie-settings-form.tsx`
- Modify: `src/app/settings/settings-view.tsx`

- [ ] **Step 1: Write failing Rust settings tests**

Add these tests to the existing `#[cfg(test)] mod tests` in `src-tauri/src/eie/config.rs`:

```rust
#[test]
fn default_settings_include_llmfit_defaults() {
    let settings = default_settings();

    assert_eq!(settings.llmfit_binary_path, None);
    assert_eq!(settings.llmfit_port, 8787);
    assert!(!settings.auto_start_llmfit);
}

#[test]
fn validation_rejects_llmfit_port_conflicts_with_eie() {
    let mut settings = default_settings();
    settings.port = 8787;
    settings.llmfit_port = 8787;

    let error = validate_settings(&settings).unwrap_err();

    assert_eq!(error.code, "invalid_llmfit_port");
}

#[test]
fn validation_rejects_non_exe_llmfit_binary_paths() {
    let mut settings = default_settings();
    settings.llmfit_binary_path = Some(PathBuf::from(r"C:\Tools\llmfit.txt"));

    let error = validate_settings(&settings).unwrap_err();

    assert_eq!(error.code, "invalid_llmfit_binary_extension");
}
```

- [ ] **Step 2: Run tests and verify RED**

Run:

```bash
rtk cargo test --no-run
```

Expected: compile failure because `EieSettings` does not have `llmfit_binary_path`, `llmfit_port`, or `auto_start_llmfit`.

- [ ] **Step 3: Implement Rust settings fields and validation**

Update `src-tauri/src/eie/types.rs`:

```rust
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
    pub llmfit_binary_path: Option<PathBuf>,
    pub llmfit_port: u16,
    pub auto_start_llmfit: bool,
}
```

Update `default_settings()` in `src-tauri/src/eie/config.rs` with:

```rust
llmfit_binary_path: None,
llmfit_port: 8787,
auto_start_llmfit: false,
```

Add validation in `validate_settings()` after EIE port validation:

```rust
if !(1024..=65535).contains(&settings.llmfit_port) || settings.llmfit_port == settings.port {
    return Err(EieError::new(
        "invalid_llmfit_port",
        "llmfit port must be between 1024 and 65535 and cannot match the EIE port.",
    ));
}

if let Some(binary_path) = &settings.llmfit_binary_path {
    let is_exe = binary_path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("exe"));

    if !is_exe {
        return Err(EieError::new(
            "invalid_llmfit_binary_extension",
            "llmfit binary must be a Windows .exe file.",
        ));
    }
}
```

- [ ] **Step 4: Update frontend settings type and defaults**

Add these fields to `EieSettings` in `src/lib/eie/types.ts`:

```ts
llmfitBinaryPath: string | null
llmfitPort: number
autoStartLlmfit: boolean
```

Add these fields to `defaultSettings` in `src/App.tsx`:

```ts
llmfitBinaryPath: null,
llmfitPort: 8787,
autoStartLlmfit: false,
```

Add form controls in `src/components/eie/eie-settings-form.tsx` near the EIE binary settings:

```tsx
<div className="grid gap-2">
  <Label htmlFor="llmfitBinaryPath">llmfit.exe path</Label>
  <Input
    id="llmfitBinaryPath"
    value={draft.llmfitBinaryPath ?? ""}
    placeholder="C:\\Tools\\llmfit.exe"
    onChange={(event) =>
      setDraft({
        ...draft,
        llmfitBinaryPath: event.target.value.trim() || null,
      })
    }
  />
</div>
<div className="grid gap-2">
  <Label htmlFor="llmfitPort">llmfit port</Label>
  <Input
    id="llmfitPort"
    min={1024}
    max={65535}
    type="number"
    value={draft.llmfitPort}
    onChange={(event) =>
      setDraft({ ...draft, llmfitPort: Number(event.target.value) })
    }
  />
</div>
<label className="flex items-center gap-2 text-sm">
  <Checkbox
    checked={draft.autoStartLlmfit}
    onCheckedChange={(checked) =>
      setDraft({ ...draft, autoStartLlmfit: checked === true })
    }
  />
  Start llmfit when Helios opens
</label>
```

- [ ] **Step 5: Run tests and typecheck**

Run:

```bash
rtk cargo test --no-run
rtk npm run typecheck
```

Expected: both commands pass.

- [ ] **Step 6: Commit**

```bash
rtk git add -- src-tauri/src/eie/types.rs src-tauri/src/eie/config.rs src/lib/eie/types.ts src/App.tsx src/components/eie/eie-settings-form.tsx src/app/settings/settings-view.tsx
rtk git commit -m "feat: add llmfit discovery settings"
```

### Task 2: Add Discovery Domain Types And Safe Parsers

**Files:**
- Create: `src-tauri/src/discovery/mod.rs`
- Create: `src-tauri/src/discovery/types.rs`
- Create: `src-tauri/src/discovery/hf.rs`
- Create: `src-tauri/src/discovery/download.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write failing Rust parser and path-safety tests**

Create `src-tauri/src/discovery/mod.rs`:

```rust
pub mod download;
pub mod hf;
pub mod types;
```

Modify `src-tauri/src/lib.rs` before running the RED check:

```rust
pub mod discovery;
```

Create `src-tauri/src/discovery/hf.rs` with tests first:

```rust
use serde::Deserialize;

use super::types::{HfGgufFile, HfModelInfo};

pub fn gguf_files_from_model_info(model: HfModelInfo) -> Vec<HfGgufFile> {
    model
        .siblings
        .into_iter()
        .filter(|sibling| sibling.rfilename.to_lowercase().ends_with(".gguf"))
        .map(|sibling| HfGgufFile {
            repo_id: model.id.clone(),
            filename: sibling.rfilename,
            size_bytes: sibling.size,
            download_url: String::new(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_keeps_only_gguf_siblings() {
        let json = r#"{
          "id": "org/model",
          "siblings": [
            { "rfilename": "model-q4.gguf", "size": 42 },
            { "rfilename": "README.md" },
            { "rfilename": "subdir/model-q5.GGUF", "size": 99 }
          ]
        }"#;
        let model: HfModelInfo = serde_json::from_str(json).unwrap();

        let files = gguf_files_from_model_info(model);

        assert_eq!(files.len(), 2);
        assert_eq!(files[0].filename, "model-q4.gguf");
        assert_eq!(files[1].filename, "subdir/model-q5.GGUF");
    }
}
```

Create `src-tauri/src/discovery/download.rs` with tests first:

```rust
use std::path::{Path, PathBuf};

use crate::eie::types::{EieError, EieResult};

pub fn resolve_download_destination(model_dir: &Path, filename: &str) -> EieResult<PathBuf> {
    if !filename.to_lowercase().ends_with(".gguf") {
        return Err(EieError::new("invalid_download_filename", "Only GGUF files can be downloaded."));
    }

    let path = Path::new(filename);
    if path.is_absolute()
        || filename.contains("..")
        || filename.contains('\\')
        || filename.contains('/')
        || filename.contains(':')
    {
        return Err(EieError::new(
            "invalid_download_filename",
            "Download filename must be a plain GGUF filename.",
        ));
    }

    Ok(model_dir.join(filename))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn destination_accepts_plain_gguf_filename() {
        let destination = resolve_download_destination(Path::new(r"C:\Models"), "model-q4.gguf").unwrap();

        assert_eq!(destination, PathBuf::from(r"C:\Models\model-q4.gguf"));
    }

    #[test]
    fn destination_rejects_traversal() {
        let error = resolve_download_destination(Path::new(r"C:\Models"), r"..\model.gguf").unwrap_err();

        assert_eq!(error.code, "invalid_download_filename");
    }

    #[test]
    fn destination_rejects_non_gguf() {
        let error = resolve_download_destination(Path::new(r"C:\Models"), "README.md").unwrap_err();

        assert_eq!(error.code, "invalid_download_filename");
    }
}
```

- [ ] **Step 2: Run tests and verify RED**

Run:

```bash
rtk cargo test --no-run
```

Expected: compile failure because `discovery::types::{HfGgufFile,HfModelInfo}` does not exist.

- [ ] **Step 3: Implement shared discovery types and module registration**

Create `src-tauri/src/discovery/types.rs`:

```rust
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
```

Confirm `src-tauri/src/lib.rs` still contains:

```rust
pub mod discovery;
```

- [ ] **Step 4: Run tests and verify GREEN**

Run:

```bash
rtk cargo test --no-run
```

Expected: tests compile.

- [ ] **Step 5: Commit**

```bash
rtk git add -- src-tauri/src/lib.rs src-tauri/src/discovery
rtk git commit -m "feat: add discovery parser primitives"
```

### Task 3: Manage llmfit Process And Local REST Calls

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Create: `src-tauri/src/discovery/llmfit.rs`
- Create: `src-tauri/src/discovery/manager.rs`
- Create: `src-tauri/src/discovery/commands.rs`
- Modify: `src-tauri/src/discovery/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write failing manager and query tests**

Create `src-tauri/src/discovery/llmfit.rs`:

```rust
use serde_json::Value;

use crate::eie::types::{EieError, EieResult};

pub fn build_llmfit_models_url(port: u16, query: &super::types::FitModelQuery) -> String {
    format!(
        "http://127.0.0.1:{}/api/v1/models?runtime=llamacpp&include_too_tight={}&limit={}&sort={}",
        port, query.include_too_tight, query.limit, query.sort
    )
}

pub fn parse_fit_models(value: Value) -> EieResult<Vec<super::types::FitModel>> {
    if let Some(items) = value.as_array() {
        return Ok(items.iter().filter_map(super::types::FitModel::from_value).collect());
    }

    if let Some(items) = value.get("models").and_then(|models| models.as_array()) {
        return Ok(items.iter().filter_map(super::types::FitModel::from_value).collect());
    }

    Err(EieError::new("invalid_llmfit_response", "llmfit returned an unknown model list shape."))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::types::FitModelQuery;

    #[test]
    fn model_query_targets_local_llamacpp_runtime() {
        let query = FitModelQuery {
            search: Some("qwen".to_string()),
            fit: "good".to_string(),
            include_too_tight: true,
            limit: 25,
            sort: "score".to_string(),
        };

        let url = build_llmfit_models_url(8787, &query);

        assert!(url.starts_with("http://127.0.0.1:8787/api/v1/models?"));
        assert!(url.contains("runtime=llamacpp"));
        assert!(url.contains("include_too_tight=true"));
        assert!(url.contains("limit=25"));
        assert!(url.contains("sort=score"));
    }

    #[test]
    fn parser_keeps_fit_fields_from_array_response() {
        let value = serde_json::json!([
            {
                "name": "Qwen 2.5 7B",
                "provider": "Qwen",
                "fit_level": "good",
                "fit_label": "Good",
                "score": 0.82,
                "estimated_tps": 44.5,
                "best_quant": "Q4_K_M",
                "memory_required_gb": 5.2,
                "gguf_sources": ["Qwen/Qwen2.5-7B-GGUF"]
            }
        ]);

        let models = parse_fit_models(value).unwrap();

        assert_eq!(models.len(), 1);
        assert_eq!(models[0].name, "Qwen 2.5 7B");
        assert_eq!(models[0].fit_level.as_deref(), Some("good"));
        assert_eq!(models[0].gguf_sources, vec!["Qwen/Qwen2.5-7B-GGUF"]);
    }
}
```

Modify `src-tauri/src/discovery/mod.rs` before running the RED check:

```rust
pub mod download;
pub mod hf;
pub mod llmfit;
pub mod types;
```

- [ ] **Step 2: Run tests and verify RED**

Run:

```bash
rtk cargo test --no-run
```

Expected: compile failure because `FitModelQuery` and `FitModel` do not exist.

- [ ] **Step 3: Add HTTP dependency**

Modify `src-tauri/Cargo.toml`:

```toml
reqwest = { version = "0.12", default-features = false, features = ["blocking", "json", "rustls-tls"] }
```

- [ ] **Step 4: Implement fit model types**

Append these types to `src-tauri/src/discovery/types.rs`:

```rust
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
            utilization_pct: value.get("utilization_pct").and_then(|field| field.as_f64()),
            gguf_sources,
        })
    }
}

fn string_field(value: &serde_json::Value, key: &str) -> Option<String> {
    value.get(key).and_then(|field| field.as_str()).map(str::to_string)
}
```

- [ ] **Step 5: Implement manager and commands**

Create `src-tauri/src/discovery/manager.rs` with a `Mutex`-protected manager:

```rust
use std::process::Child;
use std::sync::Mutex;
use std::time::Duration;

use tauri::{AppHandle, Emitter};

use crate::discovery::llmfit::{build_llmfit_models_url, parse_fit_models};
use crate::discovery::types::{FitModel, FitModelQuery, LlmfitRuntimeState, LlmfitStatus};
use crate::eie::types::{EieError, EieResult, EieSettings};

pub struct LlmfitManager {
    inner: Mutex<LlmfitManagerInner>,
}

struct LlmfitManagerInner {
    status: LlmfitStatus,
    child: Option<Child>,
}

impl Default for LlmfitManager {
    fn default() -> Self {
        Self {
            inner: Mutex::new(LlmfitManagerInner {
                status: LlmfitStatus {
                    state: LlmfitRuntimeState::Stopped,
                    pid: None,
                    base_url: "http://127.0.0.1:8787".to_string(),
                    last_error: None,
                },
                child: None,
            }),
        }
    }
}

impl LlmfitManager {
    pub fn status(&self) -> LlmfitStatus {
        self.inner.lock().expect("llmfit manager poisoned").status.clone()
    }

    pub fn list_fit_models(&self, settings: &EieSettings, query: FitModelQuery) -> EieResult<Vec<FitModel>> {
        let url = build_llmfit_models_url(settings.llmfit_port, &query);
        let value = reqwest::blocking::get(url)
            .map_err(|error| EieError::new("llmfit_request_failed", error.to_string()))?
            .json::<serde_json::Value>()
            .map_err(|error| EieError::new("llmfit_response_failed", error.to_string()))?;
        parse_fit_models(value)
    }

    pub fn emit_status(&self, app: &AppHandle) -> LlmfitStatus {
        let status = self.status();
        let _ = app.emit("llmfit://status-changed", &status);
        status
    }
}
```

Create `src-tauri/src/discovery/commands.rs` with commands that load settings, validate `llmfit_binary_path`, and delegate to `LlmfitManager`. Use `std::process::Command` with:

```rust
Command::new(binary_path)
    .args(["serve", "--host", "127.0.0.1", "--port", &settings.llmfit_port.to_string()])
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
```

Register in `src-tauri/src/discovery/mod.rs`:

```rust
pub mod commands;
pub mod download;
pub mod hf;
pub mod llmfit;
pub mod manager;
pub mod types;
```

Register managed state and commands in `src-tauri/src/lib.rs`:

```rust
.manage(discovery::manager::LlmfitManager::default())
```

Add command handlers:

```rust
discovery::commands::validate_llmfit_binary,
discovery::commands::get_llmfit_status,
discovery::commands::start_llmfit,
discovery::commands::stop_llmfit,
discovery::commands::restart_llmfit,
discovery::commands::get_llmfit_system,
discovery::commands::list_fit_models
```

- [ ] **Step 6: Run tests and check**

Run:

```bash
rtk cargo test --no-run
rtk cargo check
```

Expected: both commands pass.

- [ ] **Step 7: Commit**

```bash
rtk git add -- src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/lib.rs src-tauri/src/discovery
rtk git commit -m "feat: manage llmfit discovery helper"
```

### Task 4: Add Hugging Face GGUF Resolution And Downloads

**Files:**
- Modify: `src-tauri/src/discovery/hf.rs`
- Modify: `src-tauri/src/discovery/download.rs`
- Modify: `src-tauri/src/discovery/types.rs`
- Modify: `src-tauri/src/discovery/commands.rs`

- [ ] **Step 1: Write failing URL and download job tests**

Add to `src-tauri/src/discovery/hf.rs` tests:

```rust
#[test]
fn download_url_uses_hugging_face_resolve_main() {
    let url = build_hf_download_url("Qwen/Qwen2.5-7B-GGUF", "model-q4.gguf");

    assert_eq!(
        url,
        "https://huggingface.co/Qwen/Qwen2.5-7B-GGUF/resolve/main/model-q4.gguf"
    );
}
```

Add to `src-tauri/src/discovery/download.rs` tests:

```rust
#[test]
fn temp_destination_stays_in_model_directory() {
    let destination = resolve_download_destination(Path::new(r"C:\Models"), "model-q4.gguf").unwrap();
    let temp = temp_download_path(&destination);

    assert_eq!(temp, PathBuf::from(r"C:\Models\model-q4.gguf.helios-download"));
}
```

- [ ] **Step 2: Run tests and verify RED**

Run:

```bash
rtk cargo test --no-run
```

Expected: compile failure because `build_hf_download_url` and `temp_download_path` do not exist.

- [ ] **Step 3: Implement URL and temp path helpers**

Add to `src-tauri/src/discovery/hf.rs`:

```rust
pub fn build_hf_download_url(repo_id: &str, filename: &str) -> String {
    format!("https://huggingface.co/{repo_id}/resolve/main/{filename}")
}
```

Add to `src-tauri/src/discovery/download.rs`:

```rust
pub fn temp_download_path(destination: &Path) -> PathBuf {
    let file_name = destination
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("model.gguf");
    destination.with_file_name(format!("{file_name}.helios-download"))
}
```

- [ ] **Step 4: Implement HF metadata fetch and download jobs**

Add types in `src-tauri/src/discovery/types.rs`:

```rust
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
```

Implement command behavior:

- `get_hf_gguf_files(repo_id)` fetches `https://huggingface.co/api/models/{repo_id}`, parses `HfModelInfo`, fills each file with `build_hf_download_url`.
- `download_hf_gguf(repo_id, filename)` validates EIE `modelDirectory`, resolves destination, rejects existing destination, creates a `ModelDownload`, spawns a thread, downloads with `reqwest::blocking::Client`, writes chunks to the temp file, emits progress, renames temp to final path on success.
- `cancel_model_download(job_id)` marks a job cancelled. The download loop checks cancellation between chunks and removes the temp file when cancelled.
- `get_model_downloads` returns all current jobs.

Use event names:

```rust
"model-download://progress"
"model-download://completed"
"model-download://failed"
```

- [ ] **Step 5: Run tests and check**

Run:

```bash
rtk cargo test --no-run
rtk cargo check
```

Expected: both commands pass.

- [ ] **Step 6: Commit**

```bash
rtk git add -- src-tauri/src/discovery
rtk git commit -m "feat: add secure Hugging Face GGUF downloads"
```

### Task 5: Add TypeScript Discovery Contracts

**Files:**
- Create: `src/lib/discovery/types.ts`
- Create: `src/lib/discovery/filters.ts`
- Create: `src/lib/discovery/filters.test.ts`
- Create: `src/lib/discovery/downloads.ts`
- Modify: `src/lib/tauri/commands.ts`
- Modify: `src/lib/tauri/events.ts`

- [ ] **Step 1: Write failing filter tests**

Create `src/lib/discovery/filters.test.ts`:

```ts
import { describe, expect, test } from "bun:test"

import { createDefaultFitQuery, normalizeFitQuery } from "./filters"

describe("normalizeFitQuery", () => {
  test("defaults to runnable local llama.cpp estimates", () => {
    const query = createDefaultFitQuery()

    expect(query.fit).toBe("runnable")
    expect(query.includeTooTight).toBe(false)
    expect(query.limit).toBe(50)
    expect(query.sort).toBe("score")
  })

  test("includes too tight models only when all is selected", () => {
    const query = normalizeFitQuery({
      fit: "all",
      limit: 25,
      search: "qwen",
      sort: "estimatedTps",
    })

    expect(query.includeTooTight).toBe(true)
    expect(query.search).toBe("qwen")
    expect(query.sort).toBe("estimatedTps")
  })
})
```

- [ ] **Step 2: Run tests and verify RED**

Run:

```bash
rtk bun test src/lib/discovery/filters.test.ts
```

Expected: failure because `src/lib/discovery/filters.ts` does not exist.

- [ ] **Step 3: Implement discovery TS types and filters**

Create `src/lib/discovery/types.ts`:

```ts
export type FitFilter = "runnable" | "perfect" | "good" | "marginal" | "tooTight" | "all"
export type FitSort = "score" | "estimatedTps" | "params" | "memory" | "context" | "newest"

export type FitModelQuery = {
  search?: string
  fit: FitFilter
  includeTooTight: boolean
  limit: number
  sort: FitSort
}

export type FitModel = {
  name: string
  provider?: string
  paramsB?: number
  contextLength?: number
  useCase?: string
  fitLevel?: string
  fitLabel?: string
  runModeLabel?: string
  score?: number
  estimatedTps?: number
  runtime?: string
  runtimeLabel?: string
  bestQuant?: string
  memoryRequiredGb?: number
  memoryAvailableGb?: number
  utilizationPct?: number
  ggufSources: string[]
}

export type HfGgufFile = {
  repoId: string
  filename: string
  sizeBytes?: number
  downloadUrl: string
}

export type ModelDownload = {
  id: number
  repoId: string
  filename: string
  destination: string
  receivedBytes: number
  totalBytes?: number
  status: "queued" | "running" | "completed" | "failed" | "cancelled"
  error?: string
}

export type LlmfitStatus = {
  state: "stopped" | "starting" | "ready" | "unhealthy" | "failed"
  pid: number | null
  baseUrl: string
  lastError: string | null
}
```

Create `src/lib/discovery/filters.ts`:

```ts
import type { FitFilter, FitModelQuery, FitSort } from "./types"

export function createDefaultFitQuery(): FitModelQuery {
  return {
    fit: "runnable",
    includeTooTight: false,
    limit: 50,
    sort: "score",
  }
}

export function normalizeFitQuery(input: Partial<FitModelQuery>): FitModelQuery {
  const fit: FitFilter = input.fit ?? "runnable"
  const sort: FitSort = input.sort ?? "score"
  const search = input.search?.trim()

  return {
    fit,
    includeTooTight: fit === "all" || fit === "tooTight",
    limit: Math.min(Math.max(input.limit ?? 50, 1), 100),
    search: search ? search : undefined,
    sort,
  }
}
```

Create `src/lib/discovery/downloads.ts`:

```ts
export function formatDownloadProgress(receivedBytes: number, totalBytes?: number) {
  if (!totalBytes || totalBytes <= 0) {
    return "Downloading"
  }

  const percent = Math.min(100, Math.round((receivedBytes / totalBytes) * 100))
  return `${percent}%`
}
```

- [ ] **Step 4: Add Tauri wrappers and events**

Extend `src/lib/tauri/commands.ts` with typed wrappers:

```ts
import type {
  FitModel,
  FitModelQuery,
  HfGgufFile,
  LlmfitStatus,
  ModelDownload,
} from "@/lib/discovery/types"

export function validateLlmfitBinary(path: string) {
  return trackedInvoke<boolean>("validate_llmfit_binary", { path })
}

export function getLlmfitStatus() {
  return trackedInvoke<LlmfitStatus>("get_llmfit_status")
}

export function startLlmfit() {
  return trackedInvoke<LlmfitStatus>("start_llmfit")
}

export function stopLlmfit() {
  return trackedInvoke<LlmfitStatus>("stop_llmfit")
}

export function restartLlmfit() {
  return trackedInvoke<LlmfitStatus>("restart_llmfit")
}

export function listFitModels(query: FitModelQuery) {
  return trackedInvoke<FitModel[]>("list_fit_models", { query })
}

export function getHfGgufFiles(repoId: string) {
  return trackedInvoke<HfGgufFile[]>("get_hf_gguf_files", { repoId })
}

export function downloadHfGguf(repoId: string, filename: string) {
  return trackedInvoke<ModelDownload>("download_hf_gguf", { repoId, filename })
}

export function cancelModelDownload(jobId: number) {
  return trackedInvoke<ModelDownload>("cancel_model_download", { jobId })
}

export function getModelDownloads() {
  return trackedInvoke<ModelDownload[]>("get_model_downloads")
}
```

Extend `src/lib/tauri/events.ts`:

```ts
import type { LlmfitStatus, ModelDownload } from "@/lib/discovery/types"

export async function listenToLlmfitStatus(
  handler: (status: LlmfitStatus) => void,
): Promise<EieEventUnlisten> {
  return listen<LlmfitStatus>("llmfit://status-changed", (event) => {
    handler(event.payload)
  })
}

export async function listenToModelDownloadProgress(
  handler: (download: ModelDownload) => void,
): Promise<EieEventUnlisten> {
  return listen<ModelDownload>("model-download://progress", (event) => {
    handler(event.payload)
  })
}
```

- [ ] **Step 5: Run tests and typecheck**

Run:

```bash
rtk bun test src/lib/discovery/filters.test.ts
rtk npm run typecheck
```

Expected: tests and `tsgo` pass.

- [ ] **Step 6: Commit**

```bash
rtk git add -- src/lib/discovery src/lib/tauri/commands.ts src/lib/tauri/events.ts
rtk git commit -m "feat: add discovery frontend contracts"
```

### Task 6: Add Discover Navigation And Settings Controls

**Files:**
- Modify: `src/App.tsx`
- Modify: `src/components/app-sidebar.tsx`
- Modify: `src/app/settings/settings-view.tsx`
- Modify: `src/components/eie/eie-settings-form.tsx`

- [ ] **Step 1: Add Discover route state**

Modify `src/components/app-sidebar.tsx`:

```ts
import {
  ActivityIcon,
  BotMessageSquareIcon,
  BoxesIcon,
  CompassIcon,
  Settings2Icon,
  SparklesIcon,
} from "lucide-react"
```

```ts
export type AppView = "chat" | "discover" | "models" | "settings" | "diagnostics"
```

```ts
const navItems: NavItem[] = [
  { icon: BotMessageSquareIcon, label: "Chat", view: "chat" },
  { icon: CompassIcon, label: "Discover", view: "discover" },
  { icon: BoxesIcon, label: "Models", view: "models" },
  { icon: Settings2Icon, label: "Settings", view: "settings" },
  { icon: ActivityIcon, label: "Diagnostics", view: "diagnostics" },
]
```

- [ ] **Step 2: Add setup-state Discover view**

Create `src/app/discover/discover-view.tsx`:

```tsx
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"

export function DiscoverView() {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Discover</CardTitle>
      </CardHeader>
      <CardContent className="text-sm text-muted-foreground">
        Configure llmfit in Settings to browse runnable GGUF models.
      </CardContent>
    </Card>
  )
}
```

Modify `src/App.tsx`:

```tsx
import { DiscoverView } from "@/app/discover/discover-view"
```

```ts
const viewTitles: Record<AppView, string> = {
  chat: "Chat",
  discover: "Discover",
  diagnostics: "Diagnostics",
  models: "Models",
  settings: "Settings",
}
```

```tsx
{activeView === "discover" ? <DiscoverView /> : null}
```

- [ ] **Step 3: Wire Settings actions for llmfit lifecycle**

Add props to `SettingsView`:

```ts
llmfitStatus: LlmfitStatus
onStartLlmfit(): void
onStopLlmfit(): void
onRestartLlmfit(): void
```

Add small control row in `SettingsView` next to EIE process controls:

```tsx
<div className="flex flex-wrap items-center gap-2">
  <Button size="sm" variant="outline" onClick={onStartLlmfit}>
    Start llmfit
  </Button>
  <Button size="sm" variant="secondary" onClick={onRestartLlmfit}>
    Restart llmfit
  </Button>
  <Button size="sm" variant="outline" onClick={onStopLlmfit}>
    Stop llmfit
  </Button>
  <span className="text-sm text-muted-foreground">
    llmfit: {llmfitStatus.state}
  </span>
</div>
```

- [ ] **Step 4: Run typecheck**

Run:

```bash
rtk npm run typecheck
```

Expected: `tsgo` passes.

- [ ] **Step 5: Commit**

```bash
rtk git add -- src/App.tsx src/app/discover/discover-view.tsx src/components/app-sidebar.tsx src/app/settings/settings-view.tsx src/components/eie/eie-settings-form.tsx
rtk git commit -m "feat: add discover navigation and llmfit settings"
```

### Task 7: Build Discover Data Hook And UI

**Files:**
- Create: `src/hooks/use-model-discovery.ts`
- Modify: `src/app/discover/discover-view.tsx`
- Create: `src/components/discover/discovery-toolbar.tsx`
- Create: `src/components/discover/fit-badge.tsx`
- Create: `src/components/discover/model-fit-table.tsx`
- Create: `src/components/discover/model-detail-panel.tsx`
- Create: `src/components/discover/download-progress-list.tsx`

- [ ] **Step 1: Create discovery hook**

Create `src/hooks/use-model-discovery.ts`:

```ts
import { useCallback, useState } from "react"

import { createDefaultFitQuery, normalizeFitQuery } from "@/lib/discovery/filters"
import type { FitModel, FitModelQuery, HfGgufFile, ModelDownload } from "@/lib/discovery/types"
import {
  downloadHfGguf,
  getHfGgufFiles,
  getModelDownloads,
  listFitModels,
} from "@/lib/tauri/commands"

export function useModelDiscovery() {
  const [query, setQuery] = useState<FitModelQuery>(createDefaultFitQuery())
  const [models, setModels] = useState<FitModel[]>([])
  const [selectedModel, setSelectedModel] = useState<FitModel | null>(null)
  const [ggufFiles, setGgufFiles] = useState<HfGgufFile[]>([])
  const [downloads, setDownloads] = useState<ModelDownload[]>([])
  const [error, setError] = useState<string | null>(null)
  const [isLoading, setIsLoading] = useState(false)

  const refresh = useCallback(async (nextQuery = query) => {
    setIsLoading(true)
    setError(null)
    try {
      const normalized = normalizeFitQuery(nextQuery)
      setQuery(normalized)
      setModels(await listFitModels(normalized))
      setDownloads(await getModelDownloads())
    } catch (error) {
      setError(error instanceof Error ? error.message : String(error))
    } finally {
      setIsLoading(false)
    }
  }, [query])

  const inspectModel = useCallback(async (model: FitModel) => {
    setSelectedModel(model)
    const repoId = model.ggufSources[0]
    setGgufFiles(repoId ? await getHfGgufFiles(repoId) : [])
  }, [])

  const downloadFile = useCallback(async (file: HfGgufFile) => {
    const job = await downloadHfGguf(file.repoId, file.filename)
    setDownloads((current) => [job, ...current.filter((item) => item.id !== job.id)])
  }, [])

  return {
    downloads,
    downloadFile,
    error,
    ggufFiles,
    inspectModel,
    isLoading,
    models,
    query,
    refresh,
    selectedModel,
  }
}
```

- [ ] **Step 2: Create focused UI components**

Create `fit-badge.tsx`:

```tsx
import { Badge } from "@/components/ui/badge"

export function FitBadge({ fit }: { fit?: string }) {
  const variant = fit === "good" || fit === "perfect" ? "default" : "secondary"

  return <Badge variant={variant}>{fit ?? "unknown"}</Badge>
}
```

Create `discovery-toolbar.tsx`:

```tsx
import { RefreshCcwIcon, SearchIcon } from "lucide-react"

import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import type { FitModelQuery } from "@/lib/discovery/types"

export function DiscoveryToolbar({
  isLoading,
  query,
  onQueryChange,
  onRefresh,
}: {
  isLoading: boolean
  query: FitModelQuery
  onQueryChange(query: FitModelQuery): void
  onRefresh(): void
}) {
  return (
    <div className="flex flex-col gap-2 md:flex-row md:items-center">
      <div className="relative flex-1">
        <SearchIcon className="absolute top-2.5 left-2.5 size-4 text-muted-foreground" />
        <Input
          className="pl-8"
          placeholder="Search GGUF models"
          value={query.search ?? ""}
          onChange={(event) => onQueryChange({ ...query, search: event.target.value })}
        />
      </div>
      <Select
        value={query.fit}
        onValueChange={(fit) => onQueryChange({ ...query, fit: fit as FitModelQuery["fit"] })}
      >
        <SelectTrigger className="w-full md:w-40">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="runnable">Runnable</SelectItem>
          <SelectItem value="perfect">Perfect</SelectItem>
          <SelectItem value="good">Good</SelectItem>
          <SelectItem value="marginal">Marginal</SelectItem>
          <SelectItem value="tooTight">Too tight</SelectItem>
          <SelectItem value="all">All</SelectItem>
        </SelectContent>
      </Select>
      <Button disabled={isLoading} variant="outline" onClick={onRefresh}>
        <RefreshCcwIcon className="size-4" />
        Refresh
      </Button>
    </div>
  )
}
```

Create `model-fit-table.tsx`:

```tsx
import { useAutoAnimate } from "@formkit/auto-animate/react"

import { FitBadge } from "@/components/discover/fit-badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import type { FitModel } from "@/lib/discovery/types"

export function ModelFitTable({
  models,
  selectedModel,
  onSelectModel,
}: {
  models: FitModel[]
  selectedModel: FitModel | null
  onSelectModel(model: FitModel): void
}) {
  const [bodyRef] = useAutoAnimate<HTMLTableSectionElement>({
    duration: 160,
    easing: "ease-out",
  })

  return (
    <Card>
      <CardHeader>
        <CardTitle>Runnable Candidates</CardTitle>
      </CardHeader>
      <CardContent>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Model</TableHead>
              <TableHead>Fit</TableHead>
              <TableHead>Quant</TableHead>
              <TableHead className="text-right">TPS</TableHead>
              <TableHead className="text-right">Memory</TableHead>
              <TableHead className="text-right">GGUF</TableHead>
              <TableHead className="w-24" />
            </TableRow>
          </TableHeader>
          <TableBody ref={bodyRef}>
            {models.map((model) => (
              <TableRow
                key={`${model.provider ?? "unknown"}-${model.name}`}
                data-state={selectedModel?.name === model.name ? "selected" : undefined}
              >
                <TableCell className="max-w-[22rem]">
                  <div className="truncate font-medium">{model.name}</div>
                  <div className="truncate text-xs text-muted-foreground">
                    {model.provider ?? "Unknown provider"}
                  </div>
                </TableCell>
                <TableCell>
                  <FitBadge fit={model.fitLevel} />
                </TableCell>
                <TableCell>{model.bestQuant ?? "-"}</TableCell>
                <TableCell className="text-right">
                  {model.estimatedTps ? model.estimatedTps.toFixed(1) : "-"}
                </TableCell>
                <TableCell className="text-right">
                  {model.memoryRequiredGb ? `${model.memoryRequiredGb.toFixed(1)} GB` : "-"}
                </TableCell>
                <TableCell className="text-right">{model.ggufSources.length}</TableCell>
                <TableCell>
                  <Button size="sm" variant="outline" onClick={() => onSelectModel(model)}>
                    View
                  </Button>
                </TableCell>
              </TableRow>
            ))}
            {models.length === 0 ? (
              <TableRow>
                <TableCell colSpan={7} className="h-24 text-center text-muted-foreground">
                  No fit-ranked models loaded.
                </TableCell>
              </TableRow>
            ) : null}
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  )
}
```

Create `model-detail-panel.tsx`:

```tsx
import { DownloadIcon } from "lucide-react"

import { FitBadge } from "@/components/discover/fit-badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import type { FitModel, HfGgufFile } from "@/lib/discovery/types"

export function ModelDetailPanel({
  files,
  model,
  onDownload,
}: {
  files: HfGgufFile[]
  model: FitModel | null
  onDownload(file: HfGgufFile): void
}) {
  if (!model) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>Model Details</CardTitle>
        </CardHeader>
        <CardContent className="text-sm text-muted-foreground">
          Select a model to inspect GGUF downloads and fit estimates.
        </CardContent>
      </Card>
    )
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle className="truncate">{model.name}</CardTitle>
      </CardHeader>
      <CardContent className="grid gap-4 text-sm">
        <div className="flex items-center justify-between gap-2">
          <span className="text-muted-foreground">Estimated fit</span>
          <FitBadge fit={model.fitLevel} />
        </div>
        <div className="grid gap-1 text-muted-foreground">
          <div>Best quant: {model.bestQuant ?? "-"}</div>
          <div>Estimated TPS: {model.estimatedTps ? model.estimatedTps.toFixed(1) : "-"}</div>
          <div>Memory: {model.memoryRequiredGb ? `${model.memoryRequiredGb.toFixed(1)} GB` : "-"}</div>
          <div>Context: {model.contextLength ?? "-"}</div>
        </div>
        <div className="rounded-md border bg-muted/30 p-2 text-xs text-muted-foreground">
          Fit is estimated from llmfit GGUF/llama.cpp compatibility. It is not measured EIE throughput.
        </div>
        <div className="grid gap-2">
          {files.map((file) => (
            <div
              key={`${file.repoId}-${file.filename}`}
              className="grid gap-2 rounded-md border p-2"
            >
              <div className="truncate font-medium">{file.filename}</div>
              <div className="truncate text-xs text-muted-foreground">{file.repoId}</div>
              <Button size="sm" onClick={() => onDownload(file)}>
                <DownloadIcon className="size-4" />
                Download
              </Button>
            </div>
          ))}
          {files.length === 0 ? (
            <div className="rounded-md border p-3 text-muted-foreground">
              No GGUF files resolved yet.
            </div>
          ) : null}
        </div>
      </CardContent>
    </Card>
  )
}
```

Create `download-progress-list.tsx`:

```tsx
import { useAutoAnimate } from "@formkit/auto-animate/react"

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { formatDownloadProgress } from "@/lib/discovery/downloads"
import type { ModelDownload } from "@/lib/discovery/types"

export function DownloadProgressList({ downloads }: { downloads: ModelDownload[] }) {
  const [listRef] = useAutoAnimate<HTMLDivElement>({
    duration: 160,
    easing: "ease-out",
  })

  return (
    <Card>
      <CardHeader>
        <CardTitle>Downloads</CardTitle>
      </CardHeader>
      <CardContent ref={listRef} className="grid gap-2">
        {downloads.map((download) => (
          <div key={download.id} className="rounded-md border p-2 text-sm">
            <div className="truncate font-medium">{download.filename}</div>
            <div className="truncate text-xs text-muted-foreground">{download.destination}</div>
            <div className="mt-1 text-xs text-muted-foreground">
              {download.status} - {formatDownloadProgress(download.receivedBytes, download.totalBytes)}
            </div>
          </div>
        ))}
        {downloads.length === 0 ? (
          <div className="text-sm text-muted-foreground">No downloads yet.</div>
        ) : null}
      </CardContent>
    </Card>
  )
}
```

- [ ] **Step 3: Replace setup-state Discover view**

Update `src/app/discover/discover-view.tsx`:

```tsx
import { useEffect } from "react"

import { DiscoveryToolbar } from "@/components/discover/discovery-toolbar"
import { DownloadProgressList } from "@/components/discover/download-progress-list"
import { ModelDetailPanel } from "@/components/discover/model-detail-panel"
import { ModelFitTable } from "@/components/discover/model-fit-table"
import { useModelDiscovery } from "@/hooks/use-model-discovery"

export function DiscoverView() {
  const discovery = useModelDiscovery()
  const refresh = discovery.refresh

  useEffect(() => {
    void refresh()
  }, [refresh])

  return (
    <div className="grid gap-4 xl:grid-cols-[minmax(0,1fr)_24rem]">
      <div className="grid gap-4">
        <DiscoveryToolbar
          isLoading={discovery.isLoading}
          query={discovery.query}
          onQueryChange={(query) => void discovery.refresh(query)}
          onRefresh={() => void discovery.refresh()}
        />
        {discovery.error ? (
          <div className="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">
            {discovery.error}
          </div>
        ) : null}
        <ModelFitTable
          models={discovery.models}
          selectedModel={discovery.selectedModel}
          onSelectModel={discovery.inspectModel}
        />
        <DownloadProgressList downloads={discovery.downloads} />
      </div>
      <ModelDetailPanel
        files={discovery.ggufFiles}
        model={discovery.selectedModel}
        onDownload={discovery.downloadFile}
      />
    </div>
  )
}
```

- [ ] **Step 4: Run typecheck and build**

Run:

```bash
rtk npm run typecheck
rtk npm run build
```

Expected: both commands pass.

- [ ] **Step 5: Commit**

```bash
rtk git add -- src/hooks/use-model-discovery.ts src/app/discover src/components/discover
rtk git commit -m "feat: build model discovery page"
```

### Task 8: Wire Download Events And EIE Handoff

**Files:**
- Modify: `src/App.tsx`
- Modify: `src/hooks/use-model-discovery.ts`
- Modify: `src/lib/tauri/events.ts`
- Modify: `src/app/discover/discover-view.tsx`

- [ ] **Step 1: Listen for download progress**

Update `useModelDiscovery` to accept a callback for local model refresh and expose `applyDownloadUpdate`:

```ts
function applyDownloadUpdate(download: ModelDownload) {
  setDownloads((current) => [
    download,
    ...current.filter((item) => item.id !== download.id),
  ])
}
```

Use `listenToModelDownloadProgress` in `DiscoverView`:

```tsx
useEffect(() => {
  let unlisten: (() => void) | undefined

  void listenToModelDownloadProgress((download) => {
    discovery.applyDownloadUpdate(download)
  }).then((nextUnlisten) => {
    unlisten = nextUnlisten
  })

  return () => {
    unlisten?.()
  }
}, [discovery.applyDownloadUpdate])
```

- [ ] **Step 2: Refresh local GGUF discovery after completed downloads**

Pass `onDownloadCompleted={refreshModels}` from `App.tsx` into `DiscoverView`. In the event handler, call it when `download.status === "completed"`.

```tsx
{activeView === "discover" ? (
  <DiscoverView onDownloadCompleted={refreshModels} />
) : null}
```

- [ ] **Step 3: Show EIE restart hint**

In `DiscoverView`, show a small `Card` when a completed download appears:

```tsx
<div className="rounded-md border bg-muted/30 px-3 py-2 text-sm">
  Download complete. Restart EIE if the server does not list the new model automatically.
</div>
```

- [ ] **Step 4: Run full verification**

Run:

```bash
rtk npm run lint
rtk npm run typecheck
rtk bun test src/lib/eie/streaming.test.ts src/lib/discovery/filters.test.ts
rtk npm run build
rtk cargo check
rtk cargo test --no-run
```

Expected: all commands pass. If `rtk cargo test` still cannot launch the Tauri-linked test executable on Windows, report that exact runtime limitation and keep `cargo test --no-run` plus `cargo check` as the Rust verification evidence.

- [ ] **Step 5: Commit**

```bash
rtk git add -- src/App.tsx src/hooks/use-model-discovery.ts src/lib/tauri/events.ts src/app/discover/discover-view.tsx
rtk git commit -m "feat: connect downloads to EIE discovery"
```

## Manual Smoke Test

Use this after all code tasks are complete:

- Configure a real `llmfit.exe` path in Settings.
- Configure an existing EIE model directory.
- Start `llmfit` from Settings and confirm `llmfit: ready`.
- Open Discover and refresh models.
- Select a model with GGUF sources.
- Resolve GGUF files in the detail panel.
- Download a small GGUF file.
- Confirm the download appears in the EIE model directory.
- Open Models and confirm local GGUF discovery lists the file.
- Restart EIE and confirm `/v1/models` reflects the new file when EIE supports rediscovery.

## Self-Review Notes

- Spec coverage: settings, `llmfit` lifecycle, fit browsing, HF GGUF resolution, secure downloads, progress events, Discover UI, EIE handoff, and verification are covered by Tasks 1-8.
- Security coverage: Rust owns process execution and download writes; the frontend gets only typed commands and events.
- Scope control: private Hugging Face repos, HF token storage, resumable downloads, bundled `llmfit` sidecar, and real EIE benchmarking are not part of this MVP plan.
