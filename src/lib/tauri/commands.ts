import { trackedInvoke } from "@/lib/tauri"
import type {
  FitModel,
  FitModelQuery,
  HfGgufFile,
  LlmfitStatus,
  ModelDownload,
} from "@/lib/discovery/types"
import type {
  DiscoveredModel,
  EieConfigPreview,
  EieLogLine,
  EieSettings,
  EieStatus,
} from "@/lib/eie/types"

export function getEieSettings() {
  return trackedInvoke<EieSettings>("get_eie_settings")
}

export function saveEieSettings(settings: EieSettings) {
  return trackedInvoke<EieSettings>("save_eie_settings", { settings })
}

export function validateEieBinary(path: string) {
  return trackedInvoke<boolean>("validate_eie_binary", { path })
}

export function discoverGgufModels(modelDirectory?: string) {
  return trackedInvoke<DiscoveredModel[]>("discover_gguf_models", {
    modelDirectory,
  })
}

export function generateEieConfig() {
  return trackedInvoke<EieConfigPreview>("generate_eie_config")
}

export function startEie() {
  return trackedInvoke<EieStatus>("start_eie")
}

export function stopEie() {
  return trackedInvoke<EieStatus>("stop_eie")
}

export function restartEie() {
  return trackedInvoke<EieStatus>("restart_eie")
}

export function getEieStatus() {
  return trackedInvoke<EieStatus>("get_eie_status")
}

export function getEieLogs() {
  return trackedInvoke<EieLogLine[]>("get_eie_logs")
}

export function clearEieLogs() {
  return trackedInvoke<void>("clear_eie_logs")
}

export function openLogDir() {
  return trackedInvoke<string>("open_log_dir")
}

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
