import { useEffect } from "react"

import type { EieLogLine, EieStatus } from "@/lib/eie/types"
import { isTauri } from "@/lib/tauri"
import { listenToEieLogs, listenToEieStatus } from "@/lib/tauri/events"

type UseEieEventsOptions = {
  onStatus(status: EieStatus): void
  onLog(line: EieLogLine): void
}

export function useEieEvents({ onStatus, onLog }: UseEieEventsOptions) {
  useEffect(() => {
    if (!isTauri()) {
      return
    }

    const unlisteners: Array<() => void> = []
    let disposed = false

    void listenToEieStatus(onStatus).then((unlisten) => {
      if (disposed) {
        unlisten()
      } else {
        unlisteners.push(unlisten)
      }
    })

    void listenToEieLogs(onLog).then((unlisten) => {
      if (disposed) {
        unlisten()
      } else {
        unlisteners.push(unlisten)
      }
    })

    return () => {
      disposed = true
      for (const unlisten of unlisteners) {
        unlisten()
      }
    }
  }, [onLog, onStatus])
}
