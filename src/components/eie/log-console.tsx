import { useAutoAnimate } from "@formkit/auto-animate/react"

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
  const [logListRef] = useAutoAnimate<HTMLDivElement>({
    duration: 120,
    easing: "ease-out",
  })

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
        <div
          ref={logListRef}
          className="h-72 overflow-auto rounded-md bg-muted p-3 font-mono text-xs"
        >
          {logs.length ? (
            logs.map((entry, index) => (
              <div
                key={`${entry.timestamp}-${entry.stream}-${index}`}
                className="whitespace-pre-wrap break-words"
              >
                <span className="text-muted-foreground">
                  [{entry.stream}]
                </span>{" "}
                {entry.line}
              </div>
            ))
          ) : (
            <div className="text-muted-foreground">No EIE logs yet.</div>
          )}
        </div>
      </CardContent>
    </Card>
  )
}
