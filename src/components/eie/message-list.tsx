import { useAutoAnimate } from "@formkit/auto-animate/react"

import type { ChatTranscriptMessage } from "@/hooks/use-streaming-chat"

export function MessageList({ messages }: { messages: ChatTranscriptMessage[] }) {
  const [messageListRef] = useAutoAnimate<HTMLDivElement>({
    duration: 160,
    easing: "ease-out",
  })

  return (
    <div className="min-h-[24rem] rounded-md border bg-muted/20 p-4">
      <div className="h-[24rem] overflow-auto">
        <div ref={messageListRef} className="flex flex-col gap-3 pr-3">
          {messages.map((message) => (
            <div
              key={message.id}
              className={
                message.role === "user"
                  ? "ml-auto max-w-[80%] rounded-md bg-primary px-3 py-2 text-primary-foreground"
                  : "mr-auto max-w-[80%] rounded-md border bg-background px-3 py-2"
              }
            >
              <div className="mb-1 text-xs font-medium uppercase opacity-70">
                {message.role}
                {message.status ? ` · ${message.status}` : ""}
              </div>
              <div className="whitespace-pre-wrap text-sm">
                {message.content || "Waiting for tokens..."}
              </div>
            </div>
          ))}
          {messages.length === 0 ? (
            <div className="flex h-[20rem] items-center justify-center text-sm text-muted-foreground">
              No messages yet.
            </div>
          ) : null}
        </div>
      </div>
    </div>
  )
}
