import { useCallback, useRef, useState } from "react"

import {
  createDefaultFitQuery,
  normalizeFitQuery,
} from "@/lib/discovery/filters"
import type {
  FitModel,
  FitModelQuery,
  HfGgufFile,
  ModelDownload,
} from "@/lib/discovery/types"
import {
  downloadHfGguf,
  getHfGgufFiles,
  getModelDownloads,
  listFitModels,
} from "@/lib/tauri/commands"

export function useModelDiscovery() {
  const initialQuery = createDefaultFitQuery()
  const queryRef = useRef<FitModelQuery>(initialQuery)
  const [query, setQuery] = useState<FitModelQuery>(initialQuery)
  const [models, setModels] = useState<FitModel[]>([])
  const [selectedModel, setSelectedModel] = useState<FitModel | null>(null)
  const [ggufFiles, setGgufFiles] = useState<HfGgufFile[]>([])
  const [downloads, setDownloads] = useState<ModelDownload[]>([])
  const [error, setError] = useState<string | null>(null)
  const [isLoading, setIsLoading] = useState(false)

  const refresh = useCallback(async (nextQuery?: FitModelQuery) => {
    setIsLoading(true)
    setError(null)

    try {
      const normalized = normalizeFitQuery(nextQuery ?? queryRef.current)
      queryRef.current = normalized
      setQuery(normalized)
      setModels(await listFitModels(normalized))
      setDownloads(await getModelDownloads())
    } catch (error) {
      setError(error instanceof Error ? error.message : String(error))
    } finally {
      setIsLoading(false)
    }
  }, [])

  const inspectModel = useCallback(async (model: FitModel) => {
    setSelectedModel(model)
    setError(null)

    try {
      const repoId = model.ggufSources[0]
      setGgufFiles(repoId ? await getHfGgufFiles(repoId) : [])
    } catch (error) {
      setGgufFiles([])
      setError(error instanceof Error ? error.message : String(error))
    }
  }, [])

  const downloadFile = useCallback(async (file: HfGgufFile) => {
    const job = await downloadHfGguf(file.repoId, file.filename)
    setDownloads((current) => [
      job,
      ...current.filter((item) => item.id !== job.id),
    ])
  }, [])

  return {
    downloads,
    downloadFile,
    error,
    ggufFiles,
    inspectModel,
    isLoading,
    models,
    query,
    refresh,
    selectedModel,
  }
}
