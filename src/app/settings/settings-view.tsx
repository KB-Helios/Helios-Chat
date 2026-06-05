import { PlayIcon, RotateCwIcon, SquareIcon } from "lucide-react"

import { EieSettingsForm } from "@/components/eie/eie-settings-form"
import { ServerStatusBadge } from "@/components/eie/server-status-badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import type { LlmfitStatus } from "@/lib/discovery/types"
import type { EieSettings, EieStatus } from "@/lib/eie/types"

export function SettingsView({
  error,
  llmfitStatus,
  settings,
  status,
  onRestart,
  onRestartLlmfit,
  onSave,
  onStart,
  onStartLlmfit,
  onStop,
  onStopLlmfit,
}: {
  error: string | null
  llmfitStatus: LlmfitStatus
  settings: EieSettings
  status: EieStatus
  onRestart(): void
  onRestartLlmfit(): void
  onSave(settings: EieSettings): void
  onStart(): void
  onStartLlmfit(): void
  onStop(): void
  onStopLlmfit(): void
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
      <Card>
        <CardHeader className="flex-row items-center justify-between">
          <CardTitle>llmfit Discovery Helper</CardTitle>
          <span className="text-sm text-muted-foreground">
            {llmfitStatus.state}
          </span>
        </CardHeader>
        <CardContent className="flex flex-wrap gap-2">
          <Button onClick={onStartLlmfit}>
            <PlayIcon className="size-4" />
            Start
          </Button>
          <Button variant="secondary" onClick={onStopLlmfit}>
            <SquareIcon className="size-4" />
            Stop
          </Button>
          <Button variant="outline" onClick={onRestartLlmfit}>
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
