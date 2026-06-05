import { Badge } from "@/components/ui/badge"
import type { EieRuntimeState } from "@/lib/eie/types"

const labels: Record<EieRuntimeState, string> = {
  failed: "Failed",
  ready: "Ready",
  starting: "Starting",
  stopped: "Stopped",
  stopping: "Stopping",
  unhealthy: "Unhealthy",
}

export function ServerStatusBadge({ state }: { state: EieRuntimeState }) {
  const variant = state === "ready" ? "default" : "secondary"

  return <Badge variant={variant}>{labels[state]}</Badge>
}
