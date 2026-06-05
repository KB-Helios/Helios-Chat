import { describe, expect, test } from "bun:test"

import { createDefaultFitQuery, normalizeFitQuery } from "./filters"

describe("normalizeFitQuery", () => {
  test("defaults to runnable local llama.cpp estimates", () => {
    const query = createDefaultFitQuery()

    expect(query.fit).toBe("runnable")
    expect(query.includeTooTight).toBe(false)
    expect(query.limit).toBe(50)
    expect(query.sort).toBe("score")
  })

  test("includes too tight models only when all is selected", () => {
    const query = normalizeFitQuery({
      fit: "all",
      limit: 25,
      search: "qwen",
      sort: "estimatedTps",
    })

    expect(query.includeTooTight).toBe(true)
    expect(query.search).toBe("qwen")
    expect(query.sort).toBe("estimatedTps")
  })
})
