import { useState, type FormEvent, type ReactNode } from "react"

import { Button } from "@/components/ui/button"
import { Checkbox } from "@/components/ui/checkbox"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import type { ConfigPreset, EieSettings } from "@/lib/eie/types"

export function EieSettingsForm({
  settings,
  onSave,
}: {
  settings: EieSettings
  onSave(settings: EieSettings): void
}) {
  const [draft, setDraft] = useState(settings)

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    onSave(draft)
  }

  return (
    <form className="grid gap-4" onSubmit={handleSubmit}>
      <Field label="EIE .exe path">
        <Input
          value={draft.binaryPath ?? ""}
          onChange={(event) =>
            setDraft({ ...draft, binaryPath: event.target.value || null })
          }
        />
      </Field>
      <Field label="Model directory">
        <Input
          value={draft.modelDirectory ?? ""}
          onChange={(event) =>
            setDraft({ ...draft, modelDirectory: event.target.value || null })
          }
        />
      </Field>
      <div className="grid gap-4 md:grid-cols-3">
        <Field label="Port">
          <Input
            min={1024}
            max={65535}
            type="number"
            value={draft.port}
            onChange={(event) =>
              setDraft({ ...draft, port: Number(event.target.value) })
            }
          />
        </Field>
        <Field label="Context length">
          <Input
            min={512}
            max={262144}
            type="number"
            value={draft.contextLength}
            onChange={(event) =>
              setDraft({ ...draft, contextLength: Number(event.target.value) })
            }
          />
        </Field>
        <Field label="GPU layers">
          <Input
            min={0}
            max={999}
            type="number"
            value={draft.gpuLayers}
            onChange={(event) =>
              setDraft({ ...draft, gpuLayers: Number(event.target.value) })
            }
          />
        </Field>
      </div>
      <div className="grid gap-4 md:grid-cols-2">
        <Field label="Preset">
          <Select
            value={draft.configPreset}
            onValueChange={(value) =>
              setDraft({ ...draft, configPreset: value as ConfigPreset })
            }
          >
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="generic">Generic</SelectItem>
              <SelectItem value="development">Development</SelectItem>
              <SelectItem value="custom">Custom</SelectItem>
            </SelectContent>
          </Select>
        </Field>
        <div className="flex items-end gap-2">
          <Checkbox
            checked={draft.autoStart}
            onCheckedChange={(checked) =>
              setDraft({ ...draft, autoStart: checked === true })
            }
          />
          <Label>Auto-start EIE</Label>
        </div>
      </div>
      <Button className="w-fit" type="submit">
        Save settings
      </Button>
    </form>
  )
}

function Field({
  children,
  label,
}: {
  children: ReactNode
  label: string
}) {
  return (
    <div className="grid gap-2">
      <Label>{label}</Label>
      {children}
    </div>
  )
}
