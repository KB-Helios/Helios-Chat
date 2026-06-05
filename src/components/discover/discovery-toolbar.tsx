import { RefreshCcwIcon, SearchIcon } from "lucide-react"

import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import type { FitModelQuery } from "@/lib/discovery/types"

export function DiscoveryToolbar({
  isLoading,
  query,
  onQueryChange,
  onRefresh,
}: {
  isLoading: boolean
  query: FitModelQuery
  onQueryChange(query: FitModelQuery): void
  onRefresh(): void
}) {
  return (
    <div className="flex flex-col gap-2 md:flex-row md:items-center">
      <div className="relative flex-1">
        <SearchIcon className="absolute top-2.5 left-2.5 size-4 text-muted-foreground" />
        <Input
          className="pl-8"
          placeholder="Search GGUF models"
          value={query.search ?? ""}
          onChange={(event) =>
            onQueryChange({ ...query, search: event.target.value })
          }
        />
      </div>
      <Select
        value={query.fit}
        onValueChange={(fit) =>
          onQueryChange({ ...query, fit: fit as FitModelQuery["fit"] })
        }
      >
        <SelectTrigger className="w-full md:w-40">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="runnable">Runnable</SelectItem>
          <SelectItem value="perfect">Perfect</SelectItem>
          <SelectItem value="good">Good</SelectItem>
          <SelectItem value="marginal">Marginal</SelectItem>
          <SelectItem value="tooTight">Too tight</SelectItem>
          <SelectItem value="all">All</SelectItem>
        </SelectContent>
      </Select>
      <Button disabled={isLoading} variant="outline" onClick={onRefresh}>
        <RefreshCcwIcon className="size-4" />
        Refresh
      </Button>
    </div>
  )
}
