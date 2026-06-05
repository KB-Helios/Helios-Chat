import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import type { EieLogLine } from "@/lib/eie/types"

export function LogConsole({
  logs,
  onClear,
  onOpen,
}: {
  logs: EieLogLine[]
  onClear(): void
  onOpen(): void
}) {
  return (
    <Card>
      <CardHeader className="flex-row items-center justify-between">
        <CardTitle>EIE Logs</CardTitle>
        <div className="flex gap-2">
          <Button size="sm" variant="outline" onClick={onOpen}>
            Open folder
          </Button>
          <Button size="sm" variant="secondary" onClick={onClear}>
            Clear
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        <pre className="h-72 overflow-auto rounded-md bg-muted p-3 text-xs">
          {logs.length
            ? logs
                .map((entry) => `[${entry.stream}] ${entry.line}`)
                .join("\n")
            : "No EIE logs yet."}
        </pre>
      </CardContent>
    </Card>
  )
}
