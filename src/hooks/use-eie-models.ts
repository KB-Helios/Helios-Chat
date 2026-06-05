import { useCallback, useState } from "react"

import { createEieClient } from "@/lib/eie/client"
import type { DiscoveredModel, EieSettings, EieStatus } from "@/lib/eie/types"
import { discoverGgufModels } from "@/lib/tauri/commands"

export function useEieModels(settings: EieSettings, status: EieStatus) {
  const [servedModels, setServedModels] = useState<string[]>([])
  const [discoveredModels, setDiscoveredModels] = useState<DiscoveredModel[]>([])
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const refreshModels = useCallback(async () => {
    setIsLoading(true)
    setError(null)

    try {
      const discovered = await discoverGgufModels(settings.modelDirectory ?? undefined)
      setDiscoveredModels(discovered)

      if (status.state === "ready") {
        const client = createEieClient({
          host: settings.host,
          port: settings.port,
        })
        const models = await client.listModels()
        setServedModels(models.data.map((model) => model.id))
      }
    } catch (error) {
      setError(error instanceof Error ? error.message : String(error))
    } finally {
      setIsLoading(false)
    }
  }, [settings.host, settings.modelDirectory, settings.port, status.state])

  return {
    discoveredModels,
    error,
    isLoading,
    refreshModels,
    servedModels,
  }
}
