import { useCallback, useEffect, useMemo, useState, type CSSProperties } from "react"

import { ChatView } from "@/app/chat/chat-view"
import { DiagnosticsView } from "@/app/diagnostics/diagnostics-view"
import { ModelsView } from "@/app/models/models-view"
import { SettingsView } from "@/app/settings/settings-view"
import { AppSidebar, type AppView } from "@/components/app-sidebar"
import { SiteHeader } from "@/components/site-header"
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar"
import { TooltipProvider } from "@/components/ui/tooltip"
import { useEieEvents } from "@/hooks/use-eie-events"
import { useEieModels } from "@/hooks/use-eie-models"
import { formatEieError } from "@/lib/eie/errors"
import type {
  EieConfigPreview,
  EieLogLine,
  EieSettings,
  EieStatus,
} from "@/lib/eie/types"
import { isTauri } from "@/lib/tauri"
import {
  clearEieLogs,
  generateEieConfig,
  getEieLogs,
  getEieSettings,
  getEieStatus,
  openLogDir,
  restartEie,
  saveEieSettings,
  startEie,
  stopEie,
} from "@/lib/tauri/commands"

const defaultSettings: EieSettings = {
  autoStart: false,
  binaryPath: null,
  binarySource: "userPath",
  configPreset: "generic",
  contextLength: 8192,
  gpuLayers: 99,
  host: "127.0.0.1",
  modelDirectory: null,
  port: 8090,
}

const defaultStatus: EieStatus = {
  baseUrl: "http://127.0.0.1:8090",
  configPath: null,
  lastError: null,
  pid: null,
  state: "stopped",
}

const viewTitles: Record<AppView, string> = {
  chat: "Chat",
  diagnostics: "Diagnostics",
  models: "Models",
  settings: "Settings",
}

export default function App() {
  const [activeView, setActiveView] = useState<AppView>("chat")
  const [appError, setAppError] = useState<string | null>(null)
  const [configPreview, setConfigPreview] = useState<EieConfigPreview | null>(
    null,
  )
  const [logs, setLogs] = useState<EieLogLine[]>([])
  const [selectedModel, setSelectedModel] = useState("")
  const [settings, setSettings] = useState(defaultSettings)
  const [status, setStatus] = useState(defaultStatus)

  const handleStatus = useCallback((nextStatus: EieStatus) => {
    setStatus(nextStatus)
  }, [])

  const handleLog = useCallback((line: EieLogLine) => {
    setLogs((current) => [...current.slice(-499), line])
  }, [])

  useEieEvents({
    onLog: handleLog,
    onStatus: handleStatus,
  })

  const { discoveredModels, error: modelError, isLoading, refreshModels, servedModels } =
    useEieModels(settings, status)

  const visibleModels = useMemo(() => {
    if (servedModels.length > 0) {
      return servedModels
    }

    return discoveredModels.map((model) => model.name)
  }, [discoveredModels, servedModels])

  const effectiveSelectedModel =
    selectedModel && visibleModels.includes(selectedModel)
      ? selectedModel
      : (visibleModels[0] ?? "")

  const loadInitialState = useCallback(async () => {
    try {
      const [nextSettings, nextStatus, nextLogs] = await Promise.all([
        getEieSettings(),
        getEieStatus(),
        getEieLogs(),
      ])
      setSettings(nextSettings)
      setStatus(nextStatus)
      setLogs(nextLogs)
    } catch (error) {
      setAppError(formatEieError(error))
    }
  }, [])

  useEffect(() => {
    if (!isTauri()) {
      return
    }

    const timer = window.setTimeout(() => {
      void loadInitialState()
    }, 0)

    return () => window.clearTimeout(timer)
  }, [loadInitialState])

  async function handleSaveSettings(nextSettings: EieSettings) {
    try {
      setSettings(await saveEieSettings(nextSettings))
      setAppError(null)
    } catch (error) {
      setAppError(formatEieError(error))
    }
  }

  async function handleStart() {
    try {
      setStatus(await startEie())
      setAppError(null)
      void refreshModels()
    } catch (error) {
      setAppError(formatEieError(error))
    }
  }

  async function handleStop() {
    try {
      setStatus(await stopEie())
      setAppError(null)
    } catch (error) {
      setAppError(formatEieError(error))
    }
  }

  async function handleRestart() {
    try {
      setStatus(await restartEie())
      setAppError(null)
      void refreshModels()
    } catch (error) {
      setAppError(formatEieError(error))
    }
  }

  async function handleRefreshConfig() {
    try {
      setConfigPreview(await generateEieConfig())
      setAppError(null)
    } catch (error) {
      setAppError(formatEieError(error))
    }
  }

  async function handleClearLogs() {
    try {
      await clearEieLogs()
      setLogs([])
    } catch (error) {
      setAppError(formatEieError(error))
    }
  }

  async function handleOpenLogDir() {
    try {
      await openLogDir()
    } catch (error) {
      setAppError(formatEieError(error))
    }
  }

  return (
    <TooltipProvider>
      <SidebarProvider
        style={
          {
            "--sidebar-width": "calc(var(--spacing) * 62)",
            "--header-height": "calc(var(--spacing) * 12)",
          } as CSSProperties
        }
      >
        <AppSidebar activeView={activeView} onViewChange={setActiveView} />
        <SidebarInset>
          <SiteHeader status={status} title={viewTitles[activeView]} />
          <main className="flex flex-1 flex-col gap-4 p-4 lg:p-6">
            {appError ? (
              <div className="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">
                {appError}
              </div>
            ) : null}
            {activeView === "chat" ? (
              <ChatView
                models={visibleModels}
                selectedModel={effectiveSelectedModel}
                settings={settings}
                status={status}
                onModelChange={setSelectedModel}
              />
            ) : null}
            {activeView === "models" ? (
              <ModelsView
                discoveredModels={discoveredModels}
                error={modelError}
                isLoading={isLoading}
                servedModels={servedModels}
                onRefresh={refreshModels}
              />
            ) : null}
            {activeView === "settings" ? (
              <SettingsView
                error={appError}
                settings={settings}
                status={status}
                onRestart={handleRestart}
                onSave={handleSaveSettings}
                onStart={handleStart}
                onStop={handleStop}
              />
            ) : null}
            {activeView === "diagnostics" ? (
              <DiagnosticsView
                configPreview={configPreview}
                logs={logs}
                status={status}
                onClearLogs={handleClearLogs}
                onOpenLogDir={handleOpenLogDir}
                onRefreshConfig={handleRefreshConfig}
              />
            ) : null}
          </main>
        </SidebarInset>
      </SidebarProvider>
    </TooltipProvider>
  )
}
