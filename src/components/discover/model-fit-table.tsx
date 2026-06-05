import { useAutoAnimate } from "@formkit/auto-animate/react"

import { FitBadge } from "@/components/discover/fit-badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import type { FitModel } from "@/lib/discovery/types"

export function ModelFitTable({
  models,
  selectedModel,
  onSelectModel,
}: {
  models: FitModel[]
  selectedModel: FitModel | null
  onSelectModel(model: FitModel): void
}) {
  const [bodyRef] = useAutoAnimate<HTMLTableSectionElement>({
    duration: 160,
    easing: "ease-out",
  })

  return (
    <Card>
      <CardHeader>
        <CardTitle>Runnable Candidates</CardTitle>
      </CardHeader>
      <CardContent>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Model</TableHead>
              <TableHead>Fit</TableHead>
              <TableHead>Quant</TableHead>
              <TableHead className="text-right">TPS</TableHead>
              <TableHead className="text-right">Memory</TableHead>
              <TableHead className="text-right">GGUF</TableHead>
              <TableHead className="w-24" />
            </TableRow>
          </TableHeader>
          <TableBody ref={bodyRef}>
            {models.map((model) => (
              <TableRow
                key={`${model.provider ?? "unknown"}-${model.name}`}
                data-state={
                  selectedModel?.name === model.name ? "selected" : undefined
                }
              >
                <TableCell className="max-w-[22rem]">
                  <div className="truncate font-medium">{model.name}</div>
                  <div className="truncate text-xs text-muted-foreground">
                    {model.provider ?? "Unknown provider"}
                  </div>
                </TableCell>
                <TableCell>
                  <FitBadge fit={model.fitLevel} />
                </TableCell>
                <TableCell>{model.bestQuant ?? "-"}</TableCell>
                <TableCell className="text-right">
                  {model.estimatedTps ? model.estimatedTps.toFixed(1) : "-"}
                </TableCell>
                <TableCell className="text-right">
                  {model.memoryRequiredGb
                    ? `${model.memoryRequiredGb.toFixed(1)} GB`
                    : "-"}
                </TableCell>
                <TableCell className="text-right">
                  {model.ggufSources.length}
                </TableCell>
                <TableCell>
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={() => onSelectModel(model)}
                  >
                    View
                  </Button>
                </TableCell>
              </TableRow>
            ))}
            {models.length === 0 ? (
              <TableRow>
                <TableCell
                  colSpan={7}
                  className="h-24 text-center text-muted-foreground"
                >
                  No fit-ranked models loaded.
                </TableCell>
              </TableRow>
            ) : null}
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  )
}
