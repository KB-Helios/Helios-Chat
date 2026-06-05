import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { ChatComposer } from "@/components/eie/chat-composer"
import { MessageList } from "@/components/eie/message-list"
import { ModelPicker } from "@/components/eie/model-picker"
import { useStreamingChat } from "@/hooks/use-streaming-chat"
import type { EieSettings, EieStatus } from "@/lib/eie/types"

export function ChatView({
  models,
  selectedModel,
  settings,
  status,
  onModelChange,
}: {
  models: string[]
  selectedModel: string
  settings: EieSettings
  status: EieStatus
  onModelChange(model: string): void
}) {
  const { isStreaming, messages, sendMessage, stopStreaming } =
    useStreamingChat(settings)
  const canChat = status.state === "ready" && selectedModel.length > 0

  return (
    <div className="grid gap-4">
      <div className="flex flex-col gap-3 md:flex-row md:items-center">
        <ModelPicker
          models={models}
          value={selectedModel}
          onValueChange={onModelChange}
        />
      </div>
      <Card>
        <CardHeader>
          <CardTitle>Chat</CardTitle>
        </CardHeader>
        <CardContent className="grid gap-4">
          <MessageList messages={messages} />
          <ChatComposer
            disabled={!canChat}
            isStreaming={isStreaming}
            onSend={(message) => sendMessage(selectedModel, message)}
            onStop={stopStreaming}
          />
        </CardContent>
      </Card>
    </div>
  )
}
