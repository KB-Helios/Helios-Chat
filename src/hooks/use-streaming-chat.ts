import { useCallback, useMemo, useRef, useState } from "react"

import { createEieClient } from "@/lib/eie/client"
import type { ChatMessage, EieSettings } from "@/lib/eie/types"

export type ChatTranscriptMessage = ChatMessage & {
  id: string
  status?: "streaming" | "error" | "done"
}

export function useStreamingChat(settings: EieSettings) {
  const [messages, setMessages] = useState<ChatTranscriptMessage[]>([])
  const [isStreaming, setIsStreaming] = useState(false)
  const abortController = useRef<AbortController | null>(null)

  const client = useMemo(
    () => createEieClient({ host: settings.host, port: settings.port }),
    [settings.host, settings.port],
  )

  const sendMessage = useCallback(
    (model: string, content: string) => {
      const trimmed = content.trim()
      if (!trimmed || isStreaming) {
        return
      }

      const userMessage: ChatTranscriptMessage = {
        id: crypto.randomUUID(),
        role: "user",
        content: trimmed,
      }
      const assistantId = crypto.randomUUID()
      const assistantMessage: ChatTranscriptMessage = {
        id: assistantId,
        role: "assistant",
        content: "",
        status: "streaming",
      }

      setMessages((current) => [...current, userMessage, assistantMessage])
      setIsStreaming(true)

      abortController.current = client.streamChat(
        {
          model,
          messages: [...messages, userMessage].map(({ role, content }) => ({
            role,
            content,
          })),
        },
        {
          onDone() {
            setMessages((current) =>
              current.map((message) =>
                message.id === assistantId
                  ? { ...message, status: "done" }
                  : message,
              ),
            )
            setIsStreaming(false)
            abortController.current = null
          },
          onError(error) {
            setMessages((current) =>
              current.map((message) =>
                message.id === assistantId
                  ? {
                      ...message,
                      content: message.content || error.message,
                      status: "error",
                    }
                  : message,
              ),
            )
            setIsStreaming(false)
            abortController.current = null
          },
          onToken(token) {
            setMessages((current) =>
              current.map((message) =>
                message.id === assistantId
                  ? { ...message, content: message.content + token }
                  : message,
              ),
            )
          },
        },
      )
    },
    [client, isStreaming, messages],
  )

  const stopStreaming = useCallback(() => {
    abortController.current?.abort()
    abortController.current = null
    setIsStreaming(false)
  }, [])

  return {
    isStreaming,
    messages,
    sendMessage,
    setMessages,
    stopStreaming,
  }
}
