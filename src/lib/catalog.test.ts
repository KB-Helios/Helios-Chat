import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { catalogById, defaultCatalog, formatBytes, recommendedModel } from "./catalog";

describe("model catalog", () => {
  it("selects the balanced Qwen3 4B GGUF as the first-run recommendation", () => {
    assert.equal(recommendedModel(defaultCatalog)?.id, "qwen3-4b-q4-k-m");
  });

  it("indexes catalog entries by id", () => {
    const byId = catalogById(defaultCatalog);
    assert.equal(byId["qwen3-4b-q4-k-m"].hfRepo, "ggml-org/Qwen3-4B-GGUF");
  });

  it("formats model file sizes for compact UI display", () => {
    assert.equal(formatBytes(2_500_000_000), "2.5 GB");
    assert.equal(formatBytes(840_000_000), "840 MB");
  });
});
