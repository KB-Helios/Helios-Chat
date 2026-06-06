import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { buildKnowledgeChatFields, formatSourceStatus, toggleStackSelection } from "./knowledgeState";

describe("knowledge state", () => {
  it("toggles active stacks without duplicates", () => {
    assert.deepEqual(toggleStackSelection([], "stack-1"), ["stack-1"]);
    assert.deepEqual(toggleStackSelection(["stack-1"], "stack-1"), []);
    assert.deepEqual(toggleStackSelection(["stack-1"], "stack-2"), ["stack-1", "stack-2"]);
  });

  it("formats source status for compact UI labels", () => {
    assert.equal(formatSourceStatus("indexed"), "Indexed");
    assert.equal(formatSourceStatus("extracting"), "Extracting");
    assert.equal(formatSourceStatus("failed"), "Failed");
    assert.equal(formatSourceStatus("pending"), "Pending");
  });

  it("builds chat knowledge fields only when stacks are active", () => {
    assert.deepEqual(buildKnowledgeChatFields([]), {});
    assert.deepEqual(buildKnowledgeChatFields(["stack-1"]), {
      knowledge_stack_ids: ["stack-1"],
      retrieval_options: {
        top_k: 6,
        semantic_weight: 0.65
      }
    });
  });

  it("preserves all stack ids when building chat fields with multiple stacks", () => {
    const result = buildKnowledgeChatFields(["stack-a", "stack-b", "stack-c"]);
    assert.deepEqual(result.knowledge_stack_ids, ["stack-a", "stack-b", "stack-c"]);
    assert.equal(result.retrieval_options?.top_k, 6);
  });

  it("preserves insertion order when toggling multiple stacks", () => {
    const after1 = toggleStackSelection([], "stack-z");
    const after2 = toggleStackSelection(after1, "stack-a");
    const after3 = toggleStackSelection(after2, "stack-m");
    assert.deepEqual(after3, ["stack-z", "stack-a", "stack-m"]);
  });

  it("formatSourceStatus capitalises first letter of unknown status", () => {
    assert.equal(formatSourceStatus("processing"), "Processing");
    assert.equal(formatSourceStatus("queued"), "Queued");
  });

  it("formatSourceStatus returns Unknown for empty string", () => {
    assert.equal(formatSourceStatus(""), "Unknown");
  });
});
