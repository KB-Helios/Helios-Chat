export type StreamParseEvent =
  | { type: "token"; value: string }
  | { type: "done" }
  | { type: "error"; message: string }

type OpenAIStreamPayload = {
  choices?: Array<{
    delta?: {
      content?: string
    }
  }>
}

export function parseOpenAIStreamChunk(chunk: string): StreamParseEvent[] {
  const events: StreamParseEvent[] = []

  for (const rawLine of chunk.split(/\r?\n/)) {
    const line = rawLine.trim()

    if (!line || line.startsWith(":")) {
      continue
    }

    if (!line.startsWith("data:")) {
      continue
    }

    const data = line.slice("data:".length).trim()

    if (!data) {
      continue
    }

    if (data === "[DONE]") {
      events.push({ type: "done" })
      continue
    }

    try {
      const payload = JSON.parse(data) as OpenAIStreamPayload
      const token = payload.choices?.[0]?.delta?.content

      if (token) {
        events.push({ type: "token", value: token })
      }
    } catch {
      events.push({
        type: "error",
        message: `Malformed stream chunk: ${data}`,
      })
    }
  }

  return events
}
