import { describe, expect, test } from "bun:test"

import { buildEieUrl } from "./client"
import { parseOpenAIStreamChunk } from "./streaming"

describe("buildEieUrl", () => {
  test("builds local EIE urls with v1 paths", () => {
    expect(
      buildEieUrl({ host: "127.0.0.1", port: 8090 }, "/v1/models"),
    ).toBe("http://127.0.0.1:8090/v1/models")
  })

  test("builds local EIE health url without v1 prefix", () => {
    expect(buildEieUrl({ host: "127.0.0.1", port: 9001 }, "/health")).toBe(
      "http://127.0.0.1:9001/health",
    )
  })
})

describe("parseOpenAIStreamChunk", () => {
  test("extracts token content from OpenAI style SSE chunks", () => {
    const tokens = parseOpenAIStreamChunk(
      'data: {"choices":[{"delta":{"content":"Helios"}}]}\n\n',
    )

    expect(tokens).toEqual([{ type: "token", value: "Helios" }])
  })

  test("returns done event for DONE chunks", () => {
    expect(parseOpenAIStreamChunk("data: [DONE]\n\n")).toEqual([
      { type: "done" },
    ])
  })

  test("ignores blank keepalive chunks", () => {
    expect(parseOpenAIStreamChunk("\n\n")).toEqual([])
  })

  test("reports malformed chunks without throwing", () => {
    expect(parseOpenAIStreamChunk("data: nope\n\n")).toEqual([
      { type: "error", message: "Malformed stream chunk: nope" },
    ])
  })
})
