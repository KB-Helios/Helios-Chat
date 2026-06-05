import { RefreshCcwIcon } from "lucide-react"

import { ModelDiscoveryTable } from "@/components/eie/model-discovery-table"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import type { DiscoveredModel } from "@/lib/eie/types"

export function ModelsView({
  discoveredModels,
  error,
  isLoading,
  servedModels,
  onRefresh,
}: {
  discoveredModels: DiscoveredModel[]
  error: string | null
  isLoading: boolean
  servedModels: string[]
  onRefresh(): void
}) {
  return (
    <div className="grid gap-4">
      <div className="flex justify-end">
        <Button disabled={isLoading} variant="outline" onClick={onRefresh}>
          <RefreshCcwIcon className="size-4" />
          Refresh
        </Button>
      </div>
      <Card>
        <CardHeader>
          <CardTitle>Served Models</CardTitle>
        </CardHeader>
        <CardContent className="flex flex-wrap gap-2">
          {servedModels.length
            ? servedModels.map((model) => (
                <span
                  key={model}
                  className="rounded-md border px-2 py-1 text-sm font-medium"
                >
                  {model}
                </span>
              ))
            : "No served models."}
        </CardContent>
      </Card>
      {error ? <p className="text-sm text-destructive">{error}</p> : null}
      <ModelDiscoveryTable models={discoveredModels} />
    </div>
  )
}
