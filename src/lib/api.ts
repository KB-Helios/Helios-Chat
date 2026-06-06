import { defaultCatalog, type CatalogModel } from "./catalog";

export interface ToolStatus {
  name: string;
  present: boolean;
  path?: string;
  message: string;
  install_url: string;
}

export interface EngineStatus {
  running: boolean;
  endpoint: string;
  pid?: number;
  detail: string;
}

export interface HeliosSettings {
  default_model_id?: string;
  system_prompt: string;
  temperature: number;
  top_p: number;
  max_tokens: number;
  n_ctx: number;
  kv_type_k: string;
  kv_type_v: string;
  n_gpu_layers: number;
  idle_unload_minutes: number;
  engine_port: number;
}

export interface ChatPayload {
  conversation_id?: string;
  model: string;
  messages: Array<{ role: string; content: string }>;
  temperature: number;
  top_p: number;
  max_tokens: number;
}

export interface BuildResult {
  backend: string;
  binary_path: string;
  log_path: string;
}

export const defaultSettings: HeliosSettings = {
  default_model_id: "qwen3-4b-q4-k-m",
  system_prompt: "You are Helios, a local AI assistant running through EIE.",
  temperature: 0.7,
  top_p: 0.9,
  max_tokens: 1024,
  n_ctx: 4096,
  kv_type_k: "turbo3",
  kv_type_v: "turbo3",
  n_gpu_layers: 99,
  idle_unload_minutes: 20,
  engine_port: 8090
};

let mockEngineRunning = false;

export function isTauriRuntime(): boolean {
  return "__TAURI_INTERNALS__" in window;
}

export async function invokeCommand<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauriRuntime()) {
    return mockInvoke<T>(command, args);
  }

  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(command, args);
}

export async function setupCheckPrereqs(): Promise<ToolStatus[]> {
  return invokeCommand<ToolStatus[]>("setup_check_prereqs");
}

export async function setupBuildEie(): Promise<BuildResult> {
  return invokeCommand<BuildResult>("setup_build_eie");
}

export async function engineStart(): Promise<EngineStatus> {
  return invokeCommand<EngineStatus>("engine_start");
}

export async function engineStop(): Promise<EngineStatus> {
  return invokeCommand<EngineStatus>("engine_stop");
}

export async function engineStatus(): Promise<EngineStatus> {
  return invokeCommand<EngineStatus>("engine_status");
}

export async function modelsCatalog(): Promise<CatalogModel[]> {
  return invokeCommand<CatalogModel[]>("models_catalog");
}

export async function modelsDownload(modelId: string): Promise<string> {
  return invokeCommand<string>("models_download", { modelId });
}

export async function modelsImportLocal(sourcePath: string): Promise<string> {
  return invokeCommand<string>("models_import_local", { sourcePath });
}

export async function modelsSetDefault(modelId: string): Promise<HeliosSettings> {
  return invokeCommand<HeliosSettings>("models_set_default", { modelId });
}

export async function modelsLoad(modelId: string): Promise<void> {
  return invokeCommand<void>("models_load", { modelId });
}

export async function modelsUnload(modelId: string): Promise<void> {
  return invokeCommand<void>("models_unload", { modelId });
}

export async function settingsGet(): Promise<HeliosSettings> {
  return invokeCommand<HeliosSettings>("settings_get");
}

export async function settingsUpdate(settings: HeliosSettings): Promise<HeliosSettings> {
  return invokeCommand<HeliosSettings>("settings_update", { settings });
}

async function mockInvoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  await new Promise((resolve) => window.setTimeout(resolve, 120));

  switch (command) {
    case "setup_check_prereqs":
      return [
        { name: "git", present: true, path: "git.exe", message: "Ready", install_url: "" },
        { name: "cmake", present: true, path: "cmake.exe", message: "Ready", install_url: "" },
        { name: "cl", present: false, message: "Install Visual Studio Build Tools.", install_url: "https://visualstudio.microsoft.com/downloads/" },
        { name: "nvcc", present: false, message: "Install CUDA Toolkit for GPU acceleration.", install_url: "https://developer.nvidia.com/cuda-downloads" }
      ] as T;
    case "setup_build_eie":
      return {
        backend: "cpu",
        binary_path: "AppData/Local/Helios/engine/eie-server.exe",
        log_path: "AppData/Local/Helios/logs/eie-build.log"
      } as T;
    case "engine_start":
      mockEngineRunning = true;
      return { running: true, endpoint: "http://127.0.0.1:8090", pid: 4242, detail: "EIE process started." } as T;
    case "engine_stop":
      mockEngineRunning = false;
      return { running: false, endpoint: "http://127.0.0.1:8090", detail: "EIE is not running." } as T;
    case "engine_status":
      return {
        running: mockEngineRunning,
        endpoint: "http://127.0.0.1:8090",
        pid: mockEngineRunning ? 4242 : undefined,
        detail: mockEngineRunning ? "EIE process is managed by Helios." : "EIE is not running."
      } as T;
    case "models_catalog":
      return defaultCatalog as T;
    case "models_download":
      return `models/${args?.modelId}.gguf` as T;
    case "models_import_local":
      return String(args?.sourcePath ?? "models/imported.gguf") as T;
    case "models_load":
    case "models_unload":
      return undefined as T;
    case "models_set_default":
    case "settings_update":
      return { ...defaultSettings, ...(args?.settings as object), default_model_id: (args?.modelId as string) ?? defaultSettings.default_model_id } as T;
    case "settings_get":
      return defaultSettings as T;
    default:
      throw new Error(`Unsupported mock command: ${command}`);
  }
}
