import { listen } from "@tauri-apps/api/event"

import type { LlmfitStatus, ModelDownload } from "@/lib/discovery/types"
import type { EieLogLine, EieStatus } from "@/lib/eie/types"

export type EieEventUnlisten = () => void

export async function listenToEieStatus(
  handler: (status: EieStatus) => void,
): Promise<EieEventUnlisten> {
  return listen<EieStatus>("eie://status-changed", (event) => {
    handler(event.payload)
  })
}

export async function listenToEieLogs(
  handler: (line: EieLogLine) => void,
): Promise<EieEventUnlisten> {
  return listen<EieLogLine>("eie://log-line", (event) => {
    handler(event.payload)
  })
}

export async function listenToLlmfitStatus(
  handler: (status: LlmfitStatus) => void,
): Promise<EieEventUnlisten> {
  return listen<LlmfitStatus>("llmfit://status-changed", (event) => {
    handler(event.payload)
  })
}

export async function listenToModelDownloadProgress(
  handler: (download: ModelDownload) => void,
): Promise<EieEventUnlisten> {
  return listen<ModelDownload>("model-download://progress", (event) => {
    handler(event.payload)
  })
}
