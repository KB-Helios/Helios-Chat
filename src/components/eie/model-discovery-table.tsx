import { useAutoAnimate } from "@formkit/auto-animate/react"
import { FileIcon } from "lucide-react"

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import type { DiscoveredModel } from "@/lib/eie/types"

export function ModelDiscoveryTable({ models }: { models: DiscoveredModel[] }) {
  const [tableBodyRef] = useAutoAnimate<HTMLTableSectionElement>({
    duration: 160,
    easing: "ease-out",
  })

  return (
    <Card>
      <CardHeader>
        <CardTitle>Local GGUF Files</CardTitle>
      </CardHeader>
      <CardContent>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Name</TableHead>
              <TableHead>Path</TableHead>
              <TableHead className="text-right">Size</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody ref={tableBodyRef}>
            {models.map((model) => (
              <TableRow key={model.path}>
                <TableCell className="font-medium">
                  <span className="flex items-center gap-2">
                    <FileIcon className="size-4" />
                    {model.name}
                  </span>
                </TableCell>
                <TableCell className="max-w-[28rem] truncate text-muted-foreground">
                  {model.path}
                </TableCell>
                <TableCell className="text-right">
                  {formatBytes(model.sizeBytes)}
                </TableCell>
              </TableRow>
            ))}
            {models.length === 0 ? (
              <TableRow>
                <TableCell colSpan={3} className="h-24 text-center">
                  No GGUF files found.
                </TableCell>
              </TableRow>
            ) : null}
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  )
}

function formatBytes(bytes: number) {
  if (bytes < 1024) {
    return `${bytes} B`
  }

  const units = ["KB", "MB", "GB", "TB"]
  let value = bytes / 1024
  let unitIndex = 0

  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024
    unitIndex += 1
  }

  return `${value.toFixed(1)} ${units[unitIndex]}`
}
