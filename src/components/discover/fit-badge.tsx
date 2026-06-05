import { Badge } from "@/components/ui/badge"

export function FitBadge({ fit }: { fit?: string }) {
  const variant = fit === "good" || fit === "perfect" ? "default" : "secondary"

  return <Badge variant={variant}>{fit ?? "unknown"}</Badge>
}
