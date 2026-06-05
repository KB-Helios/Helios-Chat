export type FitFilter =
  | "runnable"
  | "perfect"
  | "good"
  | "marginal"
  | "tooTight"
  | "all"

export type FitSort =
  | "score"
  | "estimatedTps"
  | "params"
  | "memory"
  | "context"
  | "newest"

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
