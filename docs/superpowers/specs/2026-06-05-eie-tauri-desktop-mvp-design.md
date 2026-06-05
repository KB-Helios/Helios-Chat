# EIE Tauri Desktop MVP Design

Date: 2026-06-05
Status: Approved design, compact Windows-only revision
Project: `helios-ui`

## Summary

Build a Windows-only Tauri 2 desktop MVP with React, TypeScript, and shadcn/ui. The app should feel close to the Ollama desktop workflow, but use EIE as local inference infrastructure through its OpenAI-compatible REST API.

React owns chat/session UX and calls EIE REST endpoints. Rust owns EIE process management, settings validation, generated config, logs, lifecycle events, and secure local execution. MVP starts with a user-configured Windows EIE `.exe`; bundled Windows sidecars are designed in as a later source.

Sources checked:

- `https://tauriui.vercel.app/docs/getting-started`
- `https://tauri.app/develop/sidecar/`
- `https://v2.tauri.app/plugin/http-client/`
- `https://github.com/KB01111/EIE`
- `https://devblogs.microsoft.com/typescript/announcing-typescript-7-0-beta/`

## Key Decisions

- EIE is infrastructure only. App-specific orchestration, chat sessions, and UX stay in the desktop app.
- MVP targets Windows only. Linux and macOS support are out of scope.
- Default EIE host is fixed to `127.0.0.1`; generated config must not bind `0.0.0.0`.
- React never spawns binaries or receives broad shell permissions.
- Rust spawns EIE with Windows-safe argument arrays, captures stdout/stderr, polls `/health`, and stops the child process on app exit.
- MVP supports `EieBinarySource::UserPath`; later Windows sidecar packaging uses `EieBinarySource::BundledSidecar`.
- Frontend uses normal `fetch` first; add Tauri HTTP only if local EIE CORS/WebView2 behavior requires it.
- Add TypeScript Native Preview with `npm install -D @typescript/native-preview@beta` and use `tsgo` for fast type-checking.

## Project Structure

```text
src/
  app/{chat,models,settings,diagnostics}/
  components/eie/
  hooks/
  lib/eie/{client.ts,streaming.ts,types.ts}
  lib/tauri/{commands.ts,events.ts}
src-tauri/src/
  app_config.rs
  eie/{commands.rs,config.rs,logs.rs,manager.rs,models.rs,process.rs,types.rs}
docs/superpowers/{specs,plans}/
```

Keep the existing shadcn/ui and sidebar shell. Replace demo dashboard content with the MVP views.

## Rust/Tauri Surface

Managed state:

```rust
enum EieBinarySource { UserPath(PathBuf), BundledSidecar }
enum EieRuntimeState { Stopped, Starting, Ready, Unhealthy, Stopping, Failed }
```

Commands:

- `get_eie_settings`
- `save_eie_settings`
- `validate_eie_binary`
- `discover_gguf_models`
- `generate_eie_config`
- `start_eie`
- `stop_eie`
- `restart_eie`
- `get_eie_status`
- `get_eie_logs`
- `clear_eie_logs`
- `open_log_dir`

Events:

- `eie://status-changed` with `{ state, pid, baseUrl, message, timestamp }`
- `eie://log-line` with `{ stream, line, timestamp }`
- `eie://health` with `{ healthy, latencyMs, details, timestamp }`
- `eie://error` with `{ code, message, timestamp }`

## Settings And Config

Frontend type:

```ts
type EieSettings = {
  binarySource: "userPath" | "bundledSidecar"
  binaryPath: string | null
  modelDirectory: string
  host: "127.0.0.1"
  port: number
  contextLength: number
  gpuLayers: number
  configPreset: "generic" | "development" | "custom"
  autoStart: boolean
}
```

Defaults:

- `binarySource`: `userPath`
- `host`: `127.0.0.1`
- `port`: `8090`
- `contextLength`: `8192`
- `gpuLayers`: `99`
- `configPreset`: `generic`
- `autoStart`: `false`

Validation:

- Port: `1024..=65535`
- Context length: `512..=262144`
- GPU layers: `0..=999`
- Binary path must exist, be a file, and use the `.exe` extension.
- Model directory must exist and be a directory.
- Generated config path must stay under app config/data.

Generated EIE config goes to app config/data, for example `<app-config-dir>/eie/engine.generated.yaml`:

```yaml
host: 127.0.0.1
port: 8090
strategy: generic
model_dir: C:\Users\kevin\models
auto_discover: true
type_k: turbo3
type_v: turbo3
flash_attn: true
n_ctx: 8192
reserve_mb: 512
log_level: info
```

MVP launch args:

```text
--models-dir <modelDirectory> -c <contextLength> --port <port> -ngl <gpuLayers>
```

Use `--config <generatedConfigPath>` only after a smoke test confirms the target EIE binary supports it.

## Lifecycle

Start:

1. Validate settings.
2. Generate config.
3. Spawn configured Windows EIE `.exe` with explicit args.
4. Capture stdout/stderr into memory and app logs.
5. Poll `http://127.0.0.1:<port>/health`.
6. Emit `Ready` or `Failed`.

Stop:

1. Gracefully terminate the owned child process.
2. Wait for a bounded timeout.
3. Force-kill only the Windows child process Rust owns if needed.
4. Emit final status.

Restart is stop, regenerate config, start, then poll health.

Failure states should keep logs visible and explain missing binary, missing model directory, port conflict, health timeout, or unexpected process exit.

## Frontend MVP

Views:

- `ChatView`: model picker, messages, composer, streaming/cancel state.
- `ModelsView`: Rust `.gguf` discovery plus EIE `/v1/models`.
- `SettingsView`: binary path, model dir, port, context, GPU layers, preset, autostart.
- `DiagnosticsView`: status, health, pid, base URL, config preview, logs.

Components:

- `ServerStatusBadge`
- `HealthPanel`
- `ModelPicker`
- `ModelDiscoveryTable`
- `ChatComposer`
- `MessageList`
- `StreamingMessage`
- `EieSettingsForm`
- `LogConsole`

## EIE API Client

```ts
type EieClientOptions = { host: "127.0.0.1"; port: number; basePath?: "/v1" }

type EieClient = {
  health(): Promise<HealthStatus>
  listModels(): Promise<ModelList>
  discoverModels(): Promise<DiscoveredModel[]>
  chat(request: ChatCompletionRequest): Promise<ChatCompletionResponse>
  streamChat(
    request: ChatCompletionRequest,
    handlers: {
      onToken(token: string): void
      onDone(): void
      onError(error: Error): void
    }
  ): AbortController
}
```

Endpoints:

- `GET /health`
- `GET /v1/models`
- `GET /v1/admin/models/discover`
- `POST /v1/chat/completions`
- `POST /v1/chat/completions` with `stream: true`

Streaming parser handles OpenAI-style SSE chunks, appends `delta.content`, treats `[DONE]` as complete, and supports `AbortController`.

## Tooling

Install TypeScript Native Preview:

```bash
npm install -D @typescript/native-preview@beta
```

Recommended scripts:

```json
{
  "scripts": {
    "typecheck": "tsgo --noEmit",
    "typecheck:tsc": "tsc --noEmit",
    "build": "tsgo -b && vite build"
  }
}
```

Keep `typescript` installed unless verification proves eslint and other tools work without it. If `tsgo -b` has beta issues with project references, keep `tsgo --noEmit` and use `tsc -b && vite build` as the build fallback.

## Security And Packaging

- No frontend shell spawning.
- No broad shell permissions.
- Use Rust `Command` or narrow Tauri sidecar APIs with explicit Windows args.
- Keep app-managed EIE on `127.0.0.1`.
- Scope filesystem work to the binary path, model directory, app config, and app log directories.
- Do not expose arbitrary path read/write commands.
- User-path `.exe` ships first because EIE Windows binaries may differ by GPU backend and may be named `llama-server.exe`.
- Windows sidecar packaging later requires exact Tauri sidecar permissions, Authenticode signing, installer/update strategy, GPU-runtime notes, and a user-path override.

## Verification

Rust:

- Settings validation rejects invalid paths, non-`.exe` binaries, ports, host values, context length, and GPU layers.
- Config generation writes `host: 127.0.0.1` and expected values.
- GGUF discovery returns only `.gguf` files.
- Process manager prevents duplicate start and handles missing binary errors.

Frontend:

- `tsgo --noEmit` passes.
- Build script passes or documented fallback is used.
- API client builds correct local URLs.
- Streaming parser handles token chunks, `[DONE]`, abort, and malformed chunks.
- Settings form and chat state behave correctly.

Smoke:

- Configure a Windows EIE `.exe` and model directory.
- Start EIE and reach `/health`.
- List models.
- Send non-streaming and streaming chat.
- Cancel streaming.
- Stop EIE and confirm the child exits.
- Restart with changed port/context.

## Phases

1. Windows MVP foundation: Rust settings/config/discovery/process manager, Tauri commands/events, frontend wrappers, EIE client, streaming parser, `tsgo`, and four MVP views.
2. Usability: first-run setup, persisted chats, restart prompts, better diagnostics, richer GGUF metadata.
3. Windows packaging: bundled sidecars, exact permissions, Authenticode-signed binaries, installer/update strategy, user-path override.
4. Advanced EIE: VRAM/scheduling status, model load/unload, batch/chain/group workflows, tray lifecycle, optional autostart.
