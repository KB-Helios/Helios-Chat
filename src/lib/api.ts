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
  knowledge_stack_ids?: string[];
  retrieval_options?: RetrievalOptions;
}

export interface ChatResponse {
  conversation_id: string;
  content: string;
  citations: KnowledgeSearchResult[];
}

export interface BuildResult {
  backend: string;
  binary_path: string;
  log_path: string;
}

export interface RetrievalOptions {
  top_k: number;
  semantic_weight: number;
}

export interface ChatKnowledgeFields {
  knowledge_stack_ids?: string[];
  retrieval_options?: RetrievalOptions;
}

export interface KnowledgeStack {
  id: string;
  name: string;
  description: string;
  created_at: string;
  updated_at: string;
  source_count: number;
  indexed_source_count: number;
}

export interface KnowledgeSource {
  id: string;
  stack_id: string;
  path: string;
  title: string;
  format: string;
  status: string;
  content_hash?: string;
  indexed_at?: string;
  error?: string;
  created_at: string;
  updated_at: string;
}

export interface KnowledgeSearchResult {
  stack_id: string;
  source_id: string;
  source_title: string;
  chunk_id: string;
  content: string;
  score: number;
  lexical_score: number;
  semantic_score: number;
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
let mockKnowledgeStacks: KnowledgeStack[] = [
  {
    id: "local-research",
    name: "Local Research",
    description: "Browser preview stack",
    created_at: new Date(0).toISOString(),
    updated_at: new Date(0).toISOString(),
    source_count: 2,
    indexed_source_count: 2
  }
];
let mockKnowledgeSources: KnowledgeSource[] = [
  {
    id: "source-1",
    stack_id: "local-research",
    path: "C:/Helios/docs/local.md",
    title: "local.md",
    format: "md",
    status: "indexed",
    content_hash: "mock",
    indexed_at: new Date(0).toISOString(),
    created_at: new Date(0).toISOString(),
    updated_at: new Date(0).toISOString()
  },
  {
    id: "source-2",
    stack_id: "local-research",
    path: "C:/Helios/docs/notes.pdf",
    title: "notes.pdf",
    format: "pdf",
    status: "indexed",
    content_hash: "mock",
    indexed_at: new Date(0).toISOString(),
    created_at: new Date(0).toISOString(),
    updated_at: new Date(0).toISOString()
  }
];

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

export async function chatSend(request: ChatPayload): Promise<ChatResponse> {
  return invokeCommand<ChatResponse>("chat_send", { request });
}

export async function settingsGet(): Promise<HeliosSettings> {
  return invokeCommand<HeliosSettings>("settings_get");
}

export async function settingsUpdate(settings: HeliosSettings): Promise<HeliosSettings> {
  return invokeCommand<HeliosSettings>("settings_update", { settings });
}

export async function knowledgeStacksList(): Promise<KnowledgeStack[]> {
  return invokeCommand<KnowledgeStack[]>("knowledge_stacks_list");
}

export async function knowledgeStackCreate(name: string, description: string): Promise<KnowledgeStack> {
  return invokeCommand<KnowledgeStack>("knowledge_stack_create", { name, description });
}

export async function knowledgeStackUpdate(stackId: string, name: string, description: string): Promise<KnowledgeStack> {
  return invokeCommand<KnowledgeStack>("knowledge_stack_update", { stackId, name, description });
}

export async function knowledgeStackDelete(stackId: string): Promise<void> {
  return invokeCommand<void>("knowledge_stack_delete", { stackId });
}

export async function knowledgeSourcesList(stackId: string): Promise<KnowledgeSource[]> {
  return invokeCommand<KnowledgeSource[]>("knowledge_sources_list", { stackId });
}

export async function knowledgeSourcesAddFiles(stackId: string, paths: string[]): Promise<KnowledgeSource[]> {
  return invokeCommand<KnowledgeSource[]>("knowledge_sources_add_files", { stackId, paths });
}

export async function knowledgeSourcesAddFolder(stackId: string, folder: string): Promise<KnowledgeSource[]> {
  return invokeCommand<KnowledgeSource[]>("knowledge_sources_add_folder", { stackId, folder });
}

export async function knowledgeSourceRemove(sourceId: string): Promise<void> {
  return invokeCommand<void>("knowledge_source_remove", { sourceId });
}

export async function knowledgeStackReindex(stackId: string): Promise<KnowledgeSource[]> {
  return invokeCommand<KnowledgeSource[]>("knowledge_stack_reindex", { stackId });
}

export async function knowledgeSearch(stackIds: string[], query: string, options: RetrievalOptions): Promise<KnowledgeSearchResult[]> {
  return invokeCommand<KnowledgeSearchResult[]>("knowledge_search", { stackIds, query, options });
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
    case "chat_send":
      return {
        conversation_id: String(args?.conversation_id ?? crypto.randomUUID()),
        content: "EIE is wired as the default local engine. Complete first-run setup to replace this browser preview with real model tokens.",
        citations: (args?.request as ChatPayload | undefined)?.knowledge_stack_ids?.length
          ? mockKnowledgeResult()
          : []
      } as T;
    case "models_set_default":
    case "settings_update":
      return { ...defaultSettings, ...(args?.settings as object), default_model_id: (args?.modelId as string) ?? defaultSettings.default_model_id } as T;
    case "settings_get":
      return defaultSettings as T;
    case "knowledge_stacks_list":
      return mockKnowledgeStacks as T;
    case "knowledge_stack_create": {
      const now = new Date().toISOString();
      const stack: KnowledgeStack = {
        id: crypto.randomUUID(),
        name: String(args?.name ?? "Untitled stack"),
        description: String(args?.description ?? ""),
        created_at: now,
        updated_at: now,
        source_count: 0,
        indexed_source_count: 0
      };
      mockKnowledgeStacks = [stack, ...mockKnowledgeStacks];
      return stack as T;
    }
    case "knowledge_stack_update":
      mockKnowledgeStacks = mockKnowledgeStacks.map((stack) =>
        stack.id === args?.stackId
          ? { ...stack, name: String(args?.name ?? stack.name), description: String(args?.description ?? stack.description), updated_at: new Date().toISOString() }
          : stack
      );
      return mockKnowledgeStacks.find((stack) => stack.id === args?.stackId) as T;
    case "knowledge_stack_delete":
      mockKnowledgeStacks = mockKnowledgeStacks.filter((stack) => stack.id !== args?.stackId);
      mockKnowledgeSources = mockKnowledgeSources.filter((source) => source.stack_id !== args?.stackId);
      return undefined as T;
    case "knowledge_sources_list":
      return mockKnowledgeSources.filter((source) => source.stack_id === args?.stackId) as T;
    case "knowledge_sources_add_files":
      return addMockSources(String(args?.stackId), (args?.paths as string[]) ?? []) as T;
    case "knowledge_sources_add_folder":
      return addMockSources(String(args?.stackId), [`${args?.folder ?? "folder"}/notes.md`]) as T;
    case "knowledge_source_remove":
      mockKnowledgeSources = mockKnowledgeSources.filter((source) => source.id !== args?.sourceId);
      return undefined as T;
    case "knowledge_stack_reindex":
      mockKnowledgeSources = mockKnowledgeSources.map((source) =>
        source.stack_id === args?.stackId ? { ...source, status: "indexed", indexed_at: new Date().toISOString() } : source
      );
      return mockKnowledgeSources.filter((source) => source.stack_id === args?.stackId) as T;
    case "knowledge_search":
      return mockKnowledgeResult() as T;
    default:
      throw new Error(`Unsupported mock command: ${command}`);
  }
}

function addMockSources(stackId: string, paths: string[]): KnowledgeSource[] {
  const now = new Date().toISOString();
  const added = paths.map((path) => {
    const title = path.split(/[\\/]/).pop() ?? "source";
    return {
      id: crypto.randomUUID(),
      stack_id: stackId,
      path,
      title,
      format: title.split(".").pop() ?? "unknown",
      status: "indexed",
      content_hash: "mock",
      indexed_at: now,
      created_at: now,
      updated_at: now
    };
  });
  mockKnowledgeSources = [...added, ...mockKnowledgeSources];
  mockKnowledgeStacks = mockKnowledgeStacks.map((stack) =>
    stack.id === stackId
      ? { ...stack, source_count: stack.source_count + added.length, indexed_source_count: stack.indexed_source_count + added.length }
      : stack
  );
  return added;
}

function mockKnowledgeResult(): KnowledgeSearchResult[] {
  return [
    {
      stack_id: "local-research",
      source_id: "source-1",
      source_title: "local.md",
      chunk_id: "chunk-1",
      content: "Helios Knowledge Hub keeps private documents searchable on this machine.",
      score: 0.92,
      lexical_score: 0.85,
      semantic_score: 0.96
    }
  ];
}
