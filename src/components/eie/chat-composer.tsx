import { SendIcon, SquareIcon } from "lucide-react"
import { useState, type FormEvent } from "react"

import { Button } from "@/components/ui/button"

export function ChatComposer({
  disabled,
  isStreaming,
  onSend,
  onStop,
}: {
  disabled: boolean
  isStreaming: boolean
  onSend(message: string): void
  onStop(): void
}) {
  const [message, setMessage] = useState("")

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    onSend(message)
    setMessage("")
  }

  return (
    <form className="flex gap-2" onSubmit={handleSubmit}>
      <textarea
        className="min-h-24 flex-1 resize-none rounded-md border bg-background px-3 py-2 text-sm outline-none ring-offset-background placeholder:text-muted-foreground focus-visible:ring-2 focus-visible:ring-ring"
        disabled={disabled || isStreaming}
        placeholder="Ask EIE through its OpenAI-compatible API..."
        value={message}
        onChange={(event) => setMessage(event.target.value)}
      />
      {isStreaming ? (
        <Button type="button" variant="secondary" onClick={onStop}>
          <SquareIcon className="size-4" />
          Stop
        </Button>
      ) : (
        <Button disabled={disabled || !message.trim()} type="submit">
          <SendIcon className="size-4" />
          Send
        </Button>
      )}
    </form>
  )
}
