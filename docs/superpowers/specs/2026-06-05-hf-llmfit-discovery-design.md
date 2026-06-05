# Hugging Face And llmfit Discovery Design

Date: 2026-06-05
Status: Approved design
Project: `helios-ui`

## Summary

Add a Windows-only `Discover` page that lets Helios browse runnable local model candidates, show hardware fit guidance from `llmfit`, and download selected Hugging Face GGUF files into the configured EIE model directory.

`llmfit` is an advisory helper, not the inference backend. Helios manages EIE for inference, manages downloads itself, and presents `llmfit` output as a GGUF/llama.cpp-compatible estimate applied to EIE. The UI must not claim real EIE benchmarking until Helios performs EIE-specific measurements.

Sources checked:

- `https://github.com/AlexsJones/llmfit`
- `https://alexsjones-llmfit.mintlify.app/api/rest/endpoints`
- `https://alexsjones-llmfit.mintlify.app/guides/tui-mode`
- `https://huggingface.co/docs/hub/en/gguf`
- `https://huggingface.co/docs/hub/main/api`
- `https://huggingface.co/docs/hub/en/models-downloading`

## Key Decisions

- Helios manages a user-configured Windows `llmfit.exe` and starts `llmfit serve` on `127.0.0.1`.
- Rust owns `llmfit` process management, health checks, local REST calls, Hugging Face metadata calls, downloads, path validation, and progress events.
- React owns filters, table state, detail panels, and progress presentation.
- Downloads are restricted to the configured EIE `modelDirectory`.
- MVP uses public Hugging Face GGUF metadata and direct file downloads. Private/gated repos and HF token storage are deferred.
- The existing `Models` page remains local/server model inventory. The new `Discover` page is remote discovery plus fit/download workflow.
- The MVP does not implement a one-shot CLI fallback. If `llmfit serve` is unavailable, the UI shows setup/status remediation.

## Project Structure

```text
src/
  app/discover/discover-view.tsx
  components/discover/
  hooks/use-model-discovery.ts
  lib/discovery/{types.ts,filters.ts,downloads.ts}
  lib/tauri/{commands.ts,events.ts}
src-tauri/src/
  discovery/{commands.rs,download.rs,hf.rs,llmfit.rs,manager.rs,mod.rs,types.rs}
  app_config.rs
docs/superpowers/{specs,plans}/
```

Keep the existing shell, sidebar, shadcn/ui primitives, `tsgo` tooling, and EIE modules.

## Settings

Extend persisted settings with:

```ts
type DiscoverySettings = {
  llmfitBinaryPath: string | null
  llmfitPort: number
  autoStartLlmfit: boolean
}
```

Defaults:

- `llmfitBinaryPath`: `null`
- `llmfitPort`: `8787`
- `autoStartLlmfit`: `false`

Validation:

- `llmfitBinaryPath` must be an existing `.exe`.
- `llmfitPort` must be `1024..=65535` and cannot equal the EIE port.
- `llmfit` host is fixed to `127.0.0.1`.

## Rust/Tauri Surface

Managed state:

```rust
enum LlmfitRuntimeState { Stopped, Starting, Ready, Unhealthy, Failed }
struct DownloadJob { id, repo_id, filename, destination, progress, status }
```

Commands:

- `validate_llmfit_binary(path)`
- `get_llmfit_status`
- `start_llmfit`
- `stop_llmfit`
- `restart_llmfit`
- `get_llmfit_system`
- `search_fit_models(query)`
- `list_fit_models(filters)`
- `get_hf_gguf_files(repo_id)`
- `download_hf_gguf(repo_id, filename)`
- `cancel_model_download(job_id)`
- `get_model_downloads`

Events:

- `llmfit://status-changed`
- `model-download://progress`
- `model-download://completed`
- `model-download://failed`

## llmfit Integration

Launch:

```text
llmfit serve --host 127.0.0.1 --port <llmfitPort>
```

The app calls:

- `GET /health`
- `GET /api/v1/system`
- `GET /api/v1/models?runtime=llamacpp&include_too_tight=true&limit=<n>&sort=score`
- `GET /api/v1/models/top?runtime=llamacpp&min_fit=good&limit=<n>`

Helios parses only fields it needs and ignores unknown fields:

- `name`
- `provider`
- `parameter_count`
- `params_b`
- `context_length`
- `use_case`
- `fit_level`
- `fit_label`
- `run_mode_label`
- `score`
- `estimated_tps`
- `runtime`
- `runtime_label`
- `best_quant`
- `memory_required_gb`
- `memory_available_gb`
- `utilization_pct`
- `gguf_sources`

## Hugging Face Integration

The page starts from `llmfit` model rows, then resolves downloadable GGUFs through Hugging Face:

- Use `GET /api/models?filter=gguf&search=<query>&limit=<n>&full=true` only when the user searches beyond the current `llmfit` result set.
- Fetch repo details with `/api/models/{repo_id}`.
- Read `siblings` and keep files ending in `.gguf`.
- Build download URLs from `https://huggingface.co/{repo_id}/resolve/main/{filename}` unless a `gguf_sources` entry provides a better concrete URL.

Rust downloads with bounded progress reporting, writes to a temporary file in the EIE model directory, then atomically renames to the final `.gguf` path when complete. Existing files are not overwritten without an explicit future UI action.

## Discover Page

Controls:

- Search input.
- Fit segmented control: runnable, perfect, good, marginal, too tight, all.
- Use-case filter: general, coding, reasoning, chat, multimodal, embedding.
- GGUF availability toggle.
- Sort menu: score, TPS, params, memory, context, newest.
- Refresh buttons for `llmfit` and local GGUF discovery.

Table columns:

- Model
- Provider
- Params
- Fit
- Best quant
- Estimated TPS
- Memory
- Context
- GGUF
- Download status

Detail panel:

- Fit explanation and hardware summary.
- GGUF file candidates with size when available.
- Destination path.
- Download action.
- Notes explaining that fit is estimated from `llmfit`/GGUF compatibility, not measured EIE throughput.

## Lifecycle

First use:

1. User sets `llmfit.exe` path and EIE model directory.
2. Rust validates both paths.
3. User starts `llmfit` or enables autostart.
4. Discover page queries system and model fit rows.
5. User selects a GGUF candidate and starts a download.
6. Rust emits progress events and final status.
7. Helios refreshes local GGUF discovery and offers an EIE restart prompt.

Failure states should explain missing `llmfit.exe`, missing EIE model directory, port conflict, `llmfit` health timeout, Hugging Face metadata failure, gated/private repo, network failure, disk write failure, and insufficient disk space when detectable.

## Security And Packaging

- No broad shell permissions.
- No frontend binary spawning.
- Rust uses explicit command argument arrays for `llmfit`.
- `llmfit` binds only to `127.0.0.1`.
- Download destinations must resolve inside the configured EIE model directory.
- Reject filenames with path separators, drive prefixes, or non-`.gguf` extensions.
- Do not store Hugging Face tokens in MVP.
- Do not run downloaded models or scripts; downloaded artifacts are treated as data files.
- Bundled `llmfit.exe` sidecar packaging is out of scope for this MVP and belongs in the packaging phase.

## Verification

Rust:

- Binary validation accepts only existing `.exe` files.
- Port validation rejects conflicts with EIE.
- `llmfit` process manager prevents duplicate starts and reports missing binary.
- Hugging Face parser extracts only `.gguf` sibling files.
- Download path sanitizer rejects traversal and non-GGUF filenames.
- Download temp path stays inside the model directory.

Frontend:

- `tsgo --noEmit` passes.
- `vite build` passes.
- Fit filters create expected query params.
- Discover table renders empty, loading, error, ready, and downloading states.
- Download progress events update the correct row.

Smoke:

- Configure `llmfit.exe`.
- Start `llmfit serve` from Helios and reach `/health`.
- List fit-ranked models.
- Resolve GGUF files for a model.
- Download a small GGUF into the EIE model directory.
- Confirm local GGUF discovery sees the file.
- Restart EIE and confirm `/v1/models` updates when EIE supports rediscovery.

## Phases

1. Foundation: settings, Rust `llmfit` manager, status events, typed frontend wrappers.
2. Fit browsing: `Discover` page, `llmfit` model queries, filters, detail panel.
3. Hugging Face files: GGUF file resolution, destination preview, secure downloader, progress events.
4. EIE handoff: refresh local discovery, restart prompt, downloaded model visibility.
5. Polish: HF auth, resumable downloads, disk space checks, richer GGUF metadata, bundled signed `llmfit` sidecar, real EIE benchmark integration.
