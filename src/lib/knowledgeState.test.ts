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
});
