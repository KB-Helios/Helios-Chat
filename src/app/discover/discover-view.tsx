import { useEffect } from "react"

import { DiscoveryToolbar } from "@/components/discover/discovery-toolbar"
import { DownloadProgressList } from "@/components/discover/download-progress-list"
import { ModelDetailPanel } from "@/components/discover/model-detail-panel"
import { ModelFitTable } from "@/components/discover/model-fit-table"
import { useModelDiscovery } from "@/hooks/use-model-discovery"

export function DiscoverView() {
  const discovery = useModelDiscovery()
  const refresh = discovery.refresh

  useEffect(() => {
    void refresh()
  }, [refresh])

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
