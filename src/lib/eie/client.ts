import type {
  ChatCompletionRequest,
  ChatCompletionResponse,
  DiscoveredModel,
  HealthStatus,
  ModelList,
} from "./types"
import { parseOpenAIStreamChunk } from "./streaming"

export type EieClientOptions = {
  host: "127.0.0.1"
  port: number
}

export type StreamChatHandlers = {
  onToken(token: string): void
  onDone(): void
  onError(error: Error): void
}

export function buildEieUrl(options: EieClientOptions, path: string) {
  const normalizedPath = path.startsWith("/") ? path : `/${path}`
  return `http://${options.host}:${options.port}${normalizedPath}`
}

export function createEieClient(options: EieClientOptions) {
  async function requestJson<T>(path: string, init?: RequestInit): Promise<T> {
    const response = await fetch(buildEieUrl(options, path), {
      headers: {
        "Content-Type": "application/json",
        ...init?.headers,
      },
      ...init,
    })

    if (!response.ok) {
      throw new Error(`EIE request failed: ${response.status}`)
    }

    return (await response.json()) as T
  }

  return {
    health() {
      return requestJson<HealthStatus>("/health")
    },

    listModels() {
      return requestJson<ModelList>("/v1/models")
    },

    discoverModels() {
      return requestJson<DiscoveredModel[]>("/v1/admin/models/discover")
    },

    chat(request: ChatCompletionRequest) {
      return requestJson<ChatCompletionResponse>("/v1/chat/completions", {
        method: "POST",
        body: JSON.stringify({ ...request, stream: false }),
      })
    },

    streamChat(request: ChatCompletionRequest, handlers: StreamChatHandlers) {
      const controller = new AbortController()

      void streamChatRequest(options, request, handlers, controller)

      return controller
    },
  }
}

async function streamChatRequest(
  options: EieClientOptions,
  request: ChatCompletionRequest,
  handlers: StreamChatHandlers,
  controller: AbortController,
) {
  try {
    const response = await fetch(buildEieUrl(options, "/v1/chat/completions"), {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ ...request, stream: true }),
      signal: controller.signal,
    })

    if (!response.ok) {
      throw new Error(`EIE stream failed: ${response.status}`)
    }

    const reader = response.body?.getReader()
    if (!reader) {
      throw new Error("EIE stream response did not include a body.")
    }

    const decoder = new TextDecoder()

    while (true) {
      const { done, value } = await reader.read()

      if (done) {
        handlers.onDone()
        return
      }

      const chunk = decoder.decode(value, { stream: true })
      for (const event of parseOpenAIStreamChunk(chunk)) {
        if (event.type === "token") {
          handlers.onToken(event.value)
        } else if (event.type === "done") {
          handlers.onDone()
          controller.abort()
          return
        } else {
          handlers.onError(new Error(event.message))
        }
      }
    }
  } catch (error) {
    if (!controller.signal.aborted) {
      handlers.onError(error instanceof Error ? error : new Error(String(error)))
    }
  }
}
