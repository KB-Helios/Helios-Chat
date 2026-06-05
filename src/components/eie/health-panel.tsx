import { ActivityIcon, CpuIcon } from "lucide-react"

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import type { EieStatus } from "@/lib/eie/types"

export function HealthPanel({ status }: { status: EieStatus }) {
  return (
    <div className="grid gap-3 md:grid-cols-3">
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="flex items-center gap-2 text-sm">
            <ActivityIcon className="size-4" />
            Runtime
          </CardTitle>
        </CardHeader>
        <CardContent className="text-2xl font-semibold capitalize">
          {status.state}
        </CardContent>
      </Card>
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="flex items-center gap-2 text-sm">
            <CpuIcon className="size-4" />
            Process
          </CardTitle>
        </CardHeader>
        <CardContent className="text-2xl font-semibold">
          {status.pid ?? "No PID"}
        </CardContent>
      </Card>
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm">Endpoint</CardTitle>
        </CardHeader>
        <CardContent className="truncate text-sm font-medium">
          {status.baseUrl}
        </CardContent>
      </Card>
    </div>
  )
}
