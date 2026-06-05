import { PlayIcon, RotateCwIcon, SquareIcon } from "lucide-react"

import { EieSettingsForm } from "@/components/eie/eie-settings-form"
import { ServerStatusBadge } from "@/components/eie/server-status-badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import type { EieSettings, EieStatus } from "@/lib/eie/types"

export function SettingsView({
  error,
  settings,
  status,
  onRestart,
  onSave,
  onStart,
  onStop,
}: {
  error: string | null
  settings: EieSettings
  status: EieStatus
  onRestart(): void
  onSave(settings: EieSettings): void
  onStart(): void
  onStop(): void
}) {
  return (
    <div className="grid gap-4">
      <Card>
        <CardHeader className="flex-row items-center justify-between">
          <CardTitle>EIE Runtime</CardTitle>
          <ServerStatusBadge state={status.state} />
        </CardHeader>
        <CardContent className="flex flex-wrap gap-2">
          <Button onClick={onStart}>
            <PlayIcon className="size-4" />
            Start
          </Button>
          <Button variant="secondary" onClick={onStop}>
            <SquareIcon className="size-4" />
            Stop
          </Button>
          <Button variant="outline" onClick={onRestart}>
            <RotateCwIcon className="size-4" />
            Restart
          </Button>
        </CardContent>
      </Card>
      {error ? <p className="text-sm text-destructive">{error}</p> : null}
      <Card>
        <CardHeader>
          <CardTitle>Settings</CardTitle>
        </CardHeader>
        <CardContent>
          <EieSettingsForm settings={settings} onSave={onSave} />
        </CardContent>
      </Card>
    </div>
  )
}
