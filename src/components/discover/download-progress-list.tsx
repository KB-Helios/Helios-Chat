import { useAutoAnimate } from "@formkit/auto-animate/react"

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { formatDownloadProgress } from "@/lib/discovery/downloads"
import type { ModelDownload } from "@/lib/discovery/types"

export function DownloadProgressList({
  downloads,
}: {
  downloads: ModelDownload[]
}) {
  const [listRef] = useAutoAnimate<HTMLDivElement>({
    duration: 160,
    easing: "ease-out",
  })

  return (
    <Card>
      <CardHeader>
        <CardTitle>Downloads</CardTitle>
      </CardHeader>
      <CardContent ref={listRef} className="grid gap-2">
        {downloads.map((download) => (
          <div key={download.id} className="rounded-md border p-2 text-sm">
            <div className="truncate font-medium">{download.filename}</div>
            <div className="truncate text-xs text-muted-foreground">
              {download.destination}
            </div>
            <div className="mt-1 text-xs text-muted-foreground">
              {download.status} -{" "}
              {formatDownloadProgress(
                download.receivedBytes,
                download.totalBytes,
              )}
            </div>
          </div>
        ))}
        {downloads.length === 0 ? (
          <div className="text-sm text-muted-foreground">No downloads yet.</div>
        ) : null}
      </CardContent>
    </Card>
  )
}
