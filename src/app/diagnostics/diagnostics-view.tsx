import { RefreshCcwIcon } from "lucide-react"

import { HealthPanel } from "@/components/eie/health-panel"
import { LogConsole } from "@/components/eie/log-console"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import type { EieConfigPreview, EieLogLine, EieStatus } from "@/lib/eie/types"

export function DiagnosticsView({
  configPreview,
  logs,
  status,
  onClearLogs,
  onOpenLogDir,
  onRefreshConfig,
}: {
  configPreview: EieConfigPreview | null
  logs: EieLogLine[]
  status: EieStatus
  onClearLogs(): void
  onOpenLogDir(): void
  onRefreshConfig(): void
}) {
  return (
    <div className="grid gap-4">
      <HealthPanel status={status} />
      <Card>
        <CardHeader className="flex-row items-center justify-between">
          <CardTitle>Generated Config</CardTitle>
          <Button size="sm" variant="outline" onClick={onRefreshConfig}>
            <RefreshCcwIcon className="size-4" />
            Refresh
          </Button>
        </CardHeader>
        <CardContent>
          <pre className="max-h-72 overflow-auto rounded-md bg-muted p-3 text-xs">
            {configPreview
              ? `${configPreview.path}\n\n${configPreview.yaml}`
              : "No config preview."}
          </pre>
        </CardContent>
      </Card>
      <LogConsole logs={logs} onClear={onClearLogs} onOpen={onOpenLogDir} />
    </div>
  )
}
