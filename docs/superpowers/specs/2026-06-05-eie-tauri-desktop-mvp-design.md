# EIE Tauri Desktop MVP Design

Date: 2026-06-05
Status: Approved design, pending implementation plan
Project: `helios-ui`

## Summary

Build a cross-platform Tauri 2 desktop app with React, TypeScript, and shadcn/ui that feels similar in workflow to the Ollama desktop app while using EIE as local inference infrastructure. The desktop app owns chat sessions, orchestration, settings, model discovery UX, logs, and lifecycle control. EIE remains a local OpenAI-compatible REST server managed by the Tauri/Rust layer.

The MVP starts with a user-configured EIE binary path because EIE builds vary by operating system and GPU backend. The architecture still includes a sidecar-ready binary source abstraction so packaged sidecars can be added later without changing React-facing APIs.

## Sources Checked

- Tauri UI getting started: `https://tauriui.vercel.app/docs/getting-started`
- Tauri sidecar documentation: `https://tauri.app/develop/sidecar/`
- Tauri HTTP plugin permissions: `https://v2.tauri.app/plugin/http-client/`
- EIE README at `https://github.com/KB01111/EIE`
- TypeScript 7.0 Beta announcement: `https://devblogs.microsoft.com/typescript/announcing-typescript-7-0-beta/`

Important EIE facts from the README:

- EIE serves GGUF models through an OpenAI-compatible REST API.
- Relevant endpoints include `/v1/chat/completions`, `/v1/models`, `/health`, and `/v1/admin/models/discover`.
- Chat completions support streaming.
- EIE can start with `-m model.gguf` or `--models-dir /path/to/models`, plus `-c`, `--port`, and `-ngl`.
- Presets include `generic`, but the generic preset binds `host: 0.0.0.0`; this app must generate local-only config with `host: 127.0.0.1`.
- Windows build examples may produce `llama-server.exe`, so the executable name must not be hardcoded.

## Goals

- Manage EIE as a local process through the Tauri/Rust layer.
- Support a user-configured EIE binary for MVP.
- Prepare the same process manager for bundled sidecars later.
- Keep React focused on UI, chat state, and REST calls.
- Support chat completions, streaming, model listing, health checks, and local GGUF discovery.
- Expose settings for EIE binary path, model directory, port, context length, GPU layers, and config preset.
- Generate local EIE config securely under the app data directory.
- Capture logs and emit lifecycle events to the frontend.
- Default EIE host to `127.0.0.1` and reject broad host binding in app-managed config.
- Use TypeScript Native Preview through `tsgo` for fast frontend type-checking during implementation.

## Non-Goals For MVP

- No Ollama integration.
- No direct llama.cpp integration outside EIE.
- No remote inference providers.
- No EIE build automation inside the app.
- No model download manager.
- No advanced EIE group orchestration UI.
- No frontend binary spawning.
- No broad shell permission exposed to the webview.

## Recommended Project Structure

```text
helios-ui/
  docs/
    superpowers/
      specs/
  src/
    app/
      chat/
        chat-view.tsx
        chat-state.ts
      models/
        models-view.tsx
      settings/
        settings-view.tsx
      diagnostics/
        diagnostics-view.tsx
    components/
      eie/
        chat-composer.tsx
        health-panel.tsx
        log-console.tsx
        model-discovery-table.tsx
        model-picker.tsx
        server-status-badge.tsx
      ui/
    hooks/
      use-eie-events.ts
      use-eie-health.ts
      use-eie-models.ts
      use-streaming-chat.ts
    lib/
      eie/
        client.ts
        streaming.ts
        types.ts
      tauri/
        commands.ts
        events.ts
      tauri.ts
      utils.ts
  src-tauri/
    capabilities/
      default.json
    src/
      app_config.rs
      lib.rs
      eie/
        commands.rs
        config.rs
        logs.rs
        manager.rs
        models.rs
        process.rs
        types.rs
```

The existing shadcn/ui components and sidebar layout remain useful. Demo dashboard components should be replaced with app views once implementation begins.

## Rust/Tauri Architecture

The Rust side owns all process-sensitive behavior through a managed `EieManager`.

Core responsibilities:

- Load and save app settings.
- Validate the EIE binary path and model directory.
- Validate port, context length, GPU layers, and preset values.
- Generate an app-owned EIE config file.
- Start EIE with explicit arguments.
- Capture stdout and stderr into an in-memory ring buffer and app log files.
- Poll `/health` until the server is ready.
- Emit lifecycle events to the frontend.
- Stop EIE on user request and app exit.
- Prevent duplicate process starts.

Recommended state model:

```rust
enum EieBinarySource {
    UserPath(PathBuf),
    BundledSidecar,
}

enum EieRuntimeState {
    Stopped,
    Starting,
    Ready,
    Unhealthy,
    Stopping,
    Failed,
}
```

MVP implementation should use `UserPath`. `BundledSidecar` can be added after platform-specific EIE binaries are available and signed.

## Main Tauri Commands

Commands exposed to React:

| Command | Purpose |
| --- | --- |
| `get_eie_settings` | Return persisted settings and derived defaults. |
| `save_eie_settings` | Validate and persist settings. Restart is explicit unless the user chooses restart. |
| `validate_eie_binary` | Check that a configured path exists, is a file, and appears executable. |
| `discover_gguf_models` | Scan the configured model directory for `.gguf` files. |
| `generate_eie_config` | Generate and return the config path and preview for current settings. |
| `start_eie` | Start EIE if it is stopped. |
| `stop_eie` | Gracefully stop EIE, then force-kill after timeout if needed. |
| `restart_eie` | Stop, regenerate config, then start. |
| `get_eie_status` | Return process state, pid, base URL, readiness, and last error. |
| `get_eie_logs` | Return recent captured stdout/stderr lines. |
| `clear_eie_logs` | Clear the in-memory log buffer. |
| `open_log_dir` | Open the app log directory through the existing opener plugin. |

Frontend code must call these through typed wrappers in `src/lib/tauri/commands.ts`, using the existing `trackedInvoke` pattern.

## Tauri Events

Events emitted from Rust:

| Event | Payload |
| --- | --- |
| `eie://status-changed` | `{ state, pid, baseUrl, message, timestamp }` |
| `eie://log-line` | `{ stream, line, timestamp }` |
| `eie://health` | `{ healthy, latencyMs, details, timestamp }` |
| `eie://error` | `{ code, message, timestamp }` |

React subscribes through `useEieEvents` and updates local UI state. Events are status signals, not the source of chat tokens.

## EIE Settings

Persisted settings:

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

MVP defaults:

- `binarySource`: `userPath`
- `host`: `127.0.0.1`
- `port`: `8090`
- `contextLength`: `8192`
- `gpuLayers`: `99`
- `configPreset`: `generic`
- `autoStart`: `false`

Validation:

- Host is fixed to `127.0.0.1` for app-managed EIE.
- Port must be between `1024` and `65535` and not already occupied by another process unless that process appears to be the managed EIE instance.
- Binary path must exist and be a file.
- Model directory must exist and be a directory.
- Context length must be between `512` and `262144`.
- GPU layers must be between `0` and `999`.
- Generated config path must stay inside the app config/data directory.

## Generated EIE Config

The app should write a YAML config under the Tauri app config or app data directory, for example:

```text
<app-config-dir>/eie/engine.generated.yaml
```

Generated config shape:

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

For launch compatibility, the Rust process adapter should prefer explicit CLI arguments known from the EIE README:

```text
--models-dir <modelDirectory> -c <contextLength> --port <port> -ngl <gpuLayers>
```

For MVP, the manager passes explicit arguments. A later implementation may switch to `--config <generatedConfigPath>` only after a smoke test confirms the target EIE binary supports that flag.

## EIE Process Lifecycle

Startup flow:

1. React calls `start_eie`.
2. Rust loads settings.
3. Rust validates binary, model directory, and port.
4. Rust writes generated config.
5. Rust spawns EIE with a constructed argument list, not a shell string.
6. Rust captures stdout/stderr and writes log lines.
7. Rust polls `http://127.0.0.1:<port>/health`.
8. Rust transitions to `Ready` or `Failed`.
9. React enables model listing and chat when status is `Ready`.

Shutdown flow:

1. React calls `stop_eie`, or Tauri receives app exit.
2. Rust sends a graceful termination signal.
3. Rust waits for a bounded timeout.
4. Rust force-kills only the child process it owns if graceful shutdown fails.
5. Rust emits `Stopped` or `Failed` with the final result.

Restart flow:

1. Save settings.
2. Stop managed process if running.
3. Regenerate config.
4. Start with new settings.
5. Poll health again.

Failure handling:

- Missing binary: show setup state in Settings.
- Missing model directory: show discovery empty state and settings warning.
- Port conflict: show Diagnostics message and suggested alternate port.
- Health timeout: mark `Failed`, keep logs visible.
- Process exit: mark `Stopped` or `Failed` depending on whether the user requested stop.

## Frontend Views And Components

MVP views:

- `ChatView`: Chat sessions, model picker, message list, composer, streaming response state.
- `ModelsView`: Local GGUF discovery and EIE `/v1/models` results.
- `SettingsView`: EIE binary path, model directory, port, context length, GPU layers, preset, autostart.
- `DiagnosticsView`: Health status, lifecycle status, pid, base URL, logs, config preview.

MVP components:

- `ServerStatusBadge`: Compact status indicator in header/sidebar.
- `HealthPanel`: Readiness, latency, and last health result.
- `ModelPicker`: Uses `/v1/models` once EIE is ready.
- `ModelDiscoveryTable`: Lists discovered `.gguf` files from Rust scan and EIE admin discovery when available.
- `ChatComposer`: Prompt input and send/cancel controls.
- `MessageList`: User, assistant, and error messages.
- `StreamingMessage`: Incrementally renders streamed assistant output.
- `EieSettingsForm`: Validated settings form with restart prompt.
- `LogConsole`: Recent EIE logs with clear and open-log-dir actions.

Layout:

- Keep the existing Tauri UI sidebar shell.
- Replace dashboard demo sections with app routes or a simple tab/view state.
- Keep controls dense and desktop-like, closer to a local tool than a marketing app.

## TypeScript Native Preview Tooling

The implementation should add TypeScript Native Preview as a development dependency:

```bash
npm install -D @typescript/native-preview@beta
```

Use the `tsgo` executable for fast TypeScript checks. Keep the existing `typescript` package available unless implementation verification proves every current tool works without it, because eslint and other ecosystem tools may still import `typescript` directly.

Recommended script shape:

```json
{
  "scripts": {
    "typecheck": "tsgo --noEmit",
    "typecheck:tsc": "tsc --noEmit",
    "build": "tsgo -b && vite build"
  }
}
```

Implementation must verify `tsgo --noEmit` and the final build script against the current project. If `tsgo -b` exposes beta incompatibility with the existing project-reference setup, keep `tsgo --noEmit` as the primary type-check command and retain `tsc -b && vite build` as the build fallback until the issue is resolved.

## OpenAI-Compatible API Client Shape

React uses a typed EIE client under `src/lib/eie/client.ts`.

```ts
type EieClientOptions = {
  host: "127.0.0.1"
  port: number
  basePath?: "/v1"
}

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

Endpoint mapping:

| Client Method | Endpoint |
| --- | --- |
| `health` | `GET /health` |
| `listModels` | `GET /v1/models` |
| `discoverModels` | `GET /v1/admin/models/discover` |
| `chat` | `POST /v1/chat/completions` |
| `streamChat` | `POST /v1/chat/completions` with `stream: true` |

Transport:

- First choice: standard `fetch` from React to `http://127.0.0.1:<port>` if EIE provides compatible local CORS behavior.
- Fallback: Tauri HTTP plugin with permission scoped to `http://127.0.0.1:*` and only the required endpoints.
- Do not route OpenAI-compatible chat through Rust in MVP unless CORS or platform behavior requires it.

Streaming parser:

- Parse server-sent-event style chunks when EIE returns OpenAI-compatible streaming.
- Append `delta.content` tokens to the active assistant message.
- Use `AbortController` for stop generation.
- Treat `[DONE]` as completion.
- Surface malformed chunks as recoverable stream errors with raw log context in Diagnostics.

## Security Constraints

- React cannot spawn binaries.
- Do not add broad shell access to the webview.
- Use Rust `Command` or Tauri sidecar APIs with argument arrays, never concatenated shell commands.
- Bind generated EIE config to `127.0.0.1`.
- Reject app-managed `0.0.0.0` and non-local hosts.
- Store generated config under the app config/data directory.
- Limit file-system operations to configured binary path, configured model directory, app config, and app log directories.
- Do not expose arbitrary path read/write commands to React.
- Sanitize logs before displaying if future versions add secrets or auth tokens.
- Keep Tauri capabilities narrow. MVP can avoid the shell plugin entirely for React. If sidecars are added, allow only the named sidecar and expected args.

## Packaging Concerns

User-configured binary MVP:

- Works before EIE has stable packaged binaries for every target.
- Avoids bundling CUDA, ROCm, or CPU-specific binaries prematurely.
- Requires onboarding to choose the executable and model directory.

Bundled sidecar future:

- Package per platform and backend variant.
- Use Tauri sidecar configuration with exact binary names.
- Sign and notarize binaries where required.
- Make the UI clear about which backend variant is installed.
- Keep the user-path fallback available for custom EIE builds.

Cross-platform concerns:

- Windows may use `llama-server.exe` according to EIE README examples.
- Linux/macOS names and GPU support may differ.
- GPU runtime dependencies may be external to the app bundle.
- Ports can conflict with existing local inference servers.

## Testing And Verification Plan

Rust tests:

- Settings validation rejects invalid paths, invalid ports, and non-local host values.
- Config generation writes local-only host and expected numeric values.
- GGUF discovery returns only `.gguf` files and stable metadata.
- Process manager prevents duplicate start and handles missing binary errors.

Frontend tests:

- `tsgo --noEmit` passes with the final TypeScript configuration.
- If the build script uses `tsgo -b`, `npm run build` proves the native compiler path works end to end.
- EIE client builds correct URLs.
- Streaming parser handles token chunks, `[DONE]`, abort, and malformed chunks.
- Settings form validates required fields.
- Chat state appends streamed tokens to the active assistant message.

Manual smoke tests:

- Configure a local EIE binary and model directory.
- Start EIE from Settings or Diagnostics.
- Confirm `/health` becomes ready.
- List models.
- Send non-streaming chat completion.
- Send streaming chat completion and cancel mid-stream.
- Stop EIE and confirm the child process exits.
- Restart with changed port and context length.

## Phased Implementation Plan

Phase 1: MVP foundation

- Add Rust settings, config generation, GGUF discovery, and EIE process manager.
- Add Tauri commands and lifecycle events.
- Add typed frontend command wrappers.
- Add EIE REST client and streaming parser.
- Install `@typescript/native-preview@beta` and wire `tsgo` into frontend type-checking.
- Replace dashboard demo with Chat, Models, Settings, and Diagnostics views.
- Verify health, model listing, chat, streaming, logs, and stop/restart.

Phase 2: Usability and persistence

- Add first-run setup flow.
- Persist chat sessions locally.
- Add restart prompts when settings change.
- Improve diagnostics for port conflicts, health timeouts, and process crashes.
- Add better model metadata display for discovered GGUF files.

Phase 3: Packaging and sidecars

- Add `BundledSidecar` binary source.
- Package signed EIE sidecars per supported OS/backend.
- Add exact Tauri sidecar permissions.
- Add update strategy for sidecar binaries.
- Keep user-configured binary override.

Phase 4: Advanced EIE integration

- Surface `/v1/admin/vram/status`.
- Surface `/v1/admin/scheduling/status`.
- Add model load/unload controls.
- Add EIE batch, chain, and group execution workflows as app-owned UX.
- Add tray lifecycle controls and optional autostart.

## Implementation Boundary

The app should treat EIE as infrastructure. EIE serves completions and model/server status. The desktop app owns chat sessions, prompt UX, model selection, settings, orchestration decisions, diagnostics, and lifecycle.

This keeps the app portable, keeps EIE replaceable within the process-management boundary, and avoids tying React UI behavior to EIE internals beyond the OpenAI-compatible REST API and documented EIE admin endpoints.
