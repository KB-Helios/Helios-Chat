import { listen } from "@tauri-apps/api/event"

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
