import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"

export function DiscoverView() {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Discover</CardTitle>
      </CardHeader>
      <CardContent className="text-sm text-muted-foreground">
        Configure llmfit in Settings to browse runnable GGUF models.
      </CardContent>
    </Card>
  )
}
