import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { addUserMessage, appendAssistantToken, attachAssistantCitations, createInitialChatState, editMessage, finishAssistantMessage, startAssistantMessage } from "./chatState";

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

  it("attaches citations to the active assistant message", () => {
    const initial = createInitialChatState("Qwen3 4B");
    const withDraft = startAssistantMessage(initial, "chatcmpl-1");
    const cited = attachAssistantCitations(withDraft, [
      {
        sourceTitle: "local.md",
        content: "Private knowledge stays local.",
        score: 0.9
      }
    ]);

    assert.equal(cited.messages[0].citations?.[0].sourceTitle, "local.md");
  });

  it("returns state unchanged when no active assistant message exists", () => {
    const state = createInitialChatState("Qwen3 4B");
    const result = attachAssistantCitations(state, [
      { sourceTitle: "x.md", content: "text", score: 0.8 }
    ]);
    assert.strictEqual(result, state);
  });

  it("returns state unchanged when citations array is empty", () => {
    const initial = createInitialChatState("Qwen3 4B");
    const withDraft = startAssistantMessage(initial, "chatcmpl-empty");
    const result = attachAssistantCitations(withDraft, []);
    assert.strictEqual(result, withDraft);
  });

  it("replaces citations when attachAssistantCitations is called a second time", () => {
    const initial = createInitialChatState("Qwen3 4B");
    const withDraft = startAssistantMessage(initial, "chatcmpl-2");
    const firstCited = attachAssistantCitations(withDraft, [
      { sourceTitle: "first.md", content: "first content", score: 0.7 }
    ]);
    const secondCited = attachAssistantCitations(firstCited, [
      { sourceTitle: "second.md", content: "second content", score: 0.9 }
    ]);

    assert.equal(secondCited.messages[0].citations?.length, 1);
    assert.equal(secondCited.messages[0].citations?.[0].sourceTitle, "second.md");
  });

  it("does not add citations to non-active messages", () => {
    const initial = createInitialChatState("Qwen3 4B");
    const withUser = {
      ...initial,
      messages: [
        { id: "user-1", role: "user" as const, content: "hello", createdAt: new Date().toISOString() }
      ]
    };
    const withDraft = startAssistantMessage(withUser, "chatcmpl-3");
    const cited = attachAssistantCitations(withDraft, [
      { sourceTitle: "doc.txt", content: "content", score: 0.85 }
    ]);

    const userMsg = cited.messages.find((message) => message.id === "user-1");
    assert.equal(userMsg?.citations, undefined);
  });

  it("branches from an edited user message and removes later responses", () => {
    const initial = createInitialChatState("Qwen3 4B");
    const withUser = addUserMessage(initial, "hello");
    const userId = withUser.messages[0].id;
    const withDraft = startAssistantMessage(withUser, "assistant-1", userId);
    const finished = finishAssistantMessage(appendAssistantToken(withDraft, "old answer"));

    const edited = editMessage(finished, userId, "updated hello");

    assert.equal(edited.messages.length, 1);
    assert.equal(edited.messages[0].id, userId);
    assert.equal(edited.messages[0].content, "updated hello");
  });
});
