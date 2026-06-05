import { DownloadIcon } from "lucide-react"

import { FitBadge } from "@/components/discover/fit-badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import type { FitModel, HfGgufFile } from "@/lib/discovery/types"

export function ModelDetailPanel({
  files,
  model,
  onDownload,
}: {
  files: HfGgufFile[]
  model: FitModel | null
  onDownload(file: HfGgufFile): void
}) {
  if (!model) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>Model Details</CardTitle>
        </CardHeader>
        <CardContent className="text-sm text-muted-foreground">
          Select a model to inspect GGUF downloads and fit estimates.
        </CardContent>
      </Card>
    )
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle className="truncate">{model.name}</CardTitle>
      </CardHeader>
      <CardContent className="grid gap-4 text-sm">
        <div className="flex items-center justify-between gap-2">
          <span className="text-muted-foreground">Estimated fit</span>
          <FitBadge fit={model.fitLevel} />
        </div>
        <div className="grid gap-1 text-muted-foreground">
          <div>Best quant: {model.bestQuant ?? "-"}</div>
          <div>
            Estimated TPS:{" "}
            {model.estimatedTps ? model.estimatedTps.toFixed(1) : "-"}
          </div>
          <div>
            Memory:{" "}
            {model.memoryRequiredGb
              ? `${model.memoryRequiredGb.toFixed(1)} GB`
              : "-"}
          </div>
          <div>Context: {model.contextLength ?? "-"}</div>
        </div>
        <div className="rounded-md border bg-muted/30 p-2 text-xs text-muted-foreground">
          Fit is estimated from llmfit GGUF/llama.cpp compatibility. It is not
          measured EIE throughput.
        </div>
        <div className="grid gap-2">
          {files.map((file) => (
            <div
              key={`${file.repoId}-${file.filename}`}
              className="grid gap-2 rounded-md border p-2"
            >
              <div className="truncate font-medium">{file.filename}</div>
              <div className="truncate text-xs text-muted-foreground">
                {file.repoId}
              </div>
              <Button size="sm" onClick={() => onDownload(file)}>
                <DownloadIcon className="size-4" />
                Download
              </Button>
            </div>
          ))}
          {files.length === 0 ? (
            <div className="rounded-md border p-3 text-muted-foreground">
              No GGUF files resolved yet.
            </div>
          ) : null}
        </div>
      </CardContent>
    </Card>
  )
}
