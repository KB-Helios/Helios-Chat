import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { appendAssistantToken, createInitialChatState, finishAssistantMessage, startAssistantMessage } from "./chatState";

describe("chat state", () => {
  it("streams assistant tokens into the active draft message", () => {
    const initial = createInitialChatState("Qwen3 4B");
    const withDraft = startAssistantMessage(initial, "chatcmpl-1");
    const withHello = appendAssistantToken(withDraft, "Hello");
    const withWorld = appendAssistantToken(withHello, " world");
    const finished = finishAssistantMessage(withWorld);

    assert.equal(finished.messages.length, 1);
    assert.deepEqual({
      role: finished.messages[0].role,
      content: finished.messages[0].content,
      streaming: finished.messages[0].streaming
    }, {
      role: "assistant",
      content: "Hello world",
      streaming: false
    });
  });
});
