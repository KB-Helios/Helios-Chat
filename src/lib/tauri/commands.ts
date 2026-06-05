import { trackedInvoke } from "@/lib/tauri"
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
