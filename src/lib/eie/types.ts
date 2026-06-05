export type EieBinarySource = "userPath" | "bundledSidecar"
export type ConfigPreset = "generic" | "development" | "custom"
export type EieRuntimeState =
  | "stopped"
  | "starting"
  | "ready"
  | "unhealthy"
  | "stopping"
  | "failed"

export type EieSettings = {
  binarySource: EieBinarySource
  binaryPath: string | null
  modelDirectory: string | null
  host: "127.0.0.1"
  port: number
  contextLength: number
  gpuLayers: number
  configPreset: ConfigPreset
  autoStart: boolean
  llmfitBinaryPath: string | null
  llmfitPort: number
  autoStartLlmfit: boolean
}

export type EieStatus = {
  state: EieRuntimeState
  pid: number | null
  baseUrl: string
  configPath: string | null
  lastError: string | null
}

export type EieLogLine = {
  stream: "stdout" | "stderr" | string
  line: string
  timestamp: string
}

export type EieConfigPreview = {
  path: string
  yaml: string
}

export type DiscoveredModel = {
  name: string
  path: string
  sizeBytes: number
}

export type HealthStatus = {
  healthy: boolean
  status?: string
  details?: unknown
}

export type ModelList = {
  object?: string
  data: Array<{
    id: string
    object?: string
    created?: number
    owned_by?: string
  }>
}

export type ChatRole = "system" | "user" | "assistant"

export type ChatMessage = {
  role: ChatRole
  content: string
}

export type ChatCompletionRequest = {
  model: string
  messages: ChatMessage[]
  stream?: boolean
  temperature?: number
  max_tokens?: number
}

export type ChatCompletionResponse = {
  id?: string
  object?: string
  choices: Array<{
    message?: ChatMessage
    finish_reason?: string
  }>
}
