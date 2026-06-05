# Windows EIE Tauri MVP Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the Windows-only MVP desktop app that manages a local EIE `.exe`, exposes settings/model discovery/logs/lifecycle through Tauri, and provides a React chat UI backed by EIE's OpenAI-compatible API.

**Architecture:** Rust owns settings, config generation, `.exe` validation, process lifecycle, log capture, and Tauri commands/events. React owns the UI, local chat state, typed command wrappers, and EIE REST calls to `127.0.0.1`.

**Tech Stack:** Tauri 2, Rust 2021, React 19, TypeScript, shadcn/ui, Bun test runner, TypeScript Native Preview (`tsgo`).

---

## Files

- Modify: `package.json`, `bun.lock`
- Modify: `src/App.tsx`, `src/components/app-sidebar.tsx`, `src/components/site-header.tsx`
- Create: `src/lib/eie/{types.ts,streaming.ts,client.ts,streaming.test.ts}`
- Create: `src/lib/tauri/{commands.ts,events.ts}`
- Create: `src/hooks/{use-eie-events.ts,use-eie-models.ts,use-streaming-chat.ts}`
- Create: `src/app/{chat,models,settings,diagnostics}/*.tsx`
- Create: `src/components/eie/*.tsx`
- Modify: `src-tauri/Cargo.toml`, `src-tauri/src/lib.rs`
- Create: `src-tauri/src/app_config.rs`
- Create: `src-tauri/src/eie/{mod.rs,types.rs,config.rs,models.rs,logs.rs,process.rs,manager.rs,commands.rs}`

## Tasks

### Task 1: Tooling

- [x] Install `@typescript/native-preview@beta` with `npm install -D @typescript/native-preview@beta`.
- [x] Update scripts to use `tsgo --noEmit`, keep `typecheck:tsc`, and keep build fallback if `tsgo -b` fails.
- [x] Verify `rtk npm run typecheck` reaches TypeScript checks.

### Task 2: Rust Settings, Config, And Discovery

- [x] Add failing Rust unit tests for Windows `.exe` validation, local-only host defaults, config YAML generation, and `.gguf` discovery.
- [x] Run `rtk cargo test` in `src-tauri` and confirm the tests fail because modules are missing.
- [x] Implement `types.rs`, `config.rs`, `models.rs`, and `app_config.rs`.
- [x] Run `rtk cargo test --no-run` and `rtk cargo check` in `src-tauri`.

### Task 3: Rust Process Manager And Commands

- [x] Add failing Rust tests for duplicate-start prevention and missing-binary status behavior.
- [x] Implement `logs.rs`, `process.rs`, `manager.rs`, and `commands.rs`.
- [x] Register managed `EieManager` state and all Tauri commands in `lib.rs`.
- [x] Run `rtk cargo test --no-run` and `rtk cargo check` in `src-tauri`.

### Task 4: TypeScript Client And Streaming Parser

- [x] Add Bun tests for OpenAI-style SSE token parsing, `[DONE]`, malformed chunks, and URL construction.
- [x] Run `rtk bun test src/lib/eie/streaming.test.ts` and confirm failures before implementation.
- [x] Implement `types.ts`, `streaming.ts`, `client.ts`, and Tauri command/event wrappers.
- [x] Run `rtk bun test src/lib/eie/streaming.test.ts`.

### Task 5: MVP UI

- [x] Replace dashboard demo content with Chat, Models, Settings, and Diagnostics views.
- [x] Implement EIE components and hooks using existing shadcn/ui primitives and lucide icons.
- [x] Ensure the UI never imports shell APIs or spawns binaries.
- [x] Run `rtk npm run typecheck`.

### Task 6: Verification

- [x] Run `rtk cargo test --no-run` and `rtk cargo check` in `src-tauri`.
- [x] Run `rtk bun test src/lib/eie/streaming.test.ts`.
- [x] Run `rtk npm run typecheck`.
- [x] Run `rtk npm run build`, using the documented `tsc -b && vite build` fallback only if `tsgo -b` beta behavior blocks the build.
- [x] Report any manual smoke-test steps that require a real EIE `.exe` and GGUF model directory.
