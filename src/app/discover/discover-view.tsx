import { useEffect, useState } from "react"

import { DiscoveryToolbar } from "@/components/discover/discovery-toolbar"
import { DownloadProgressList } from "@/components/discover/download-progress-list"
import { ModelDetailPanel } from "@/components/discover/model-detail-panel"
import { ModelFitTable } from "@/components/discover/model-fit-table"
import { useModelDiscovery } from "@/hooks/use-model-discovery"
import { listenToModelDownloadProgress } from "@/lib/tauri/events"

export function DiscoverView({
  onDownloadCompleted,
}: {
  onDownloadCompleted(): void
}) {
  const discovery = useModelDiscovery()
  const applyDownloadUpdate = discovery.applyDownloadUpdate
  const refresh = discovery.refresh
  const [hasCompletedDownload, setHasCompletedDownload] = useState(false)

  useEffect(() => {
    void refresh()
  }, [refresh])

  useEffect(() => {
    let unlisten: (() => void) | undefined

    void listenToModelDownloadProgress((download) => {
      applyDownloadUpdate(download)

      if (download.status === "completed") {
        setHasCompletedDownload(true)
        onDownloadCompleted()
      }
    }).then((nextUnlisten) => {
      unlisten = nextUnlisten
    })

    return () => {
      unlisten?.()
    }
  }, [applyDownloadUpdate, onDownloadCompleted])

  return (
    <div className="grid gap-4 xl:grid-cols-[minmax(0,1fr)_24rem]">
      <div className="grid gap-4">
        <DiscoveryToolbar
          isLoading={discovery.isLoading}
          query={discovery.query}
          onQueryChange={(query) => void discovery.refresh(query)}
          onRefresh={() => void discovery.refresh()}
        />
        {discovery.error ? (
          <div className="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">
            {discovery.error}
          </div>
        ) : null}
        {hasCompletedDownload ? (
          <div className="rounded-md border bg-muted/30 px-3 py-2 text-sm">
            Download complete. Restart EIE if the server does not list the new
            model automatically.
          </div>
        ) : null}
        <ModelFitTable
          models={discovery.models}
          selectedModel={discovery.selectedModel}
          onSelectModel={discovery.inspectModel}
        />
        <DownloadProgressList downloads={discovery.downloads} />
      </div>
      <ModelDetailPanel
        files={discovery.ggufFiles}
        model={discovery.selectedModel}
        onDownload={discovery.downloadFile}
      />
    </div>
  )
}
