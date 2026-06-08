import { describe, it } from "node:test";
import assert from "node:assert/strict";
import {
  defaultSettings,
  type ChatProvider,
  type Conversation,
  type Message,
  type Preset,
  type ChatPayload,
  type ProviderKind
} from "./api";

describe("api module – defaultSettings", () => {
  it("has the expected default values", () => {
    assert.equal(defaultSettings.default_model_id, "qwen3-4b-q4-k-m");
    assert.equal(defaultSettings.temperature, 0.7);
    assert.equal(defaultSettings.top_p, 0.9);
    assert.equal(defaultSettings.max_tokens, 1024);
    assert.equal(defaultSettings.n_ctx, 4096);
    assert.equal(defaultSettings.n_gpu_layers, 99);
    assert.equal(defaultSettings.engine_port, 8090);
    assert.equal(defaultSettings.kv_type_k, "turbo3");
    assert.equal(defaultSettings.kv_type_v, "turbo3");
    assert.equal(defaultSettings.idle_unload_minutes, 20);
  });

  it("system_prompt is non-empty and mentions Helios", () => {
    assert.ok(defaultSettings.system_prompt.length > 0, "system_prompt should not be empty");
    assert.ok(
      defaultSettings.system_prompt.includes("Helios"),
      "system_prompt should mention Helios"
    );
  });

  it("temperature is within the valid range 0–1.5", () => {
    assert.ok(defaultSettings.temperature >= 0, "temperature must be >= 0");
    assert.ok(defaultSettings.temperature <= 1.5, "temperature must be <= 1.5");
  });

  it("top_p is within the valid range 0.1–1", () => {
    assert.ok(defaultSettings.top_p >= 0.1, "top_p must be >= 0.1");
    assert.ok(defaultSettings.top_p <= 1, "top_p must be <= 1");
  });

  it("max_tokens and n_ctx are positive integers", () => {
    assert.ok(Number.isInteger(defaultSettings.max_tokens), "max_tokens must be an integer");
    assert.ok(defaultSettings.max_tokens > 0, "max_tokens must be positive");
    assert.ok(Number.isInteger(defaultSettings.n_ctx), "n_ctx must be an integer");
    assert.ok(defaultSettings.n_ctx > 0, "n_ctx must be positive");
  });
});

describe("api module – ChatProvider interface contract", () => {
  it("accepts a valid ChatProvider object including all required fields", () => {
    const provider: ChatProvider = {
      id: "openai",
      kind: "openai" as ProviderKind,
      label: "OpenAI",
      enabled: true,
      requiresKey: true,
      hasKey: true,
      baseUrl: "https://api.openai.com/v1",
      models: ["gpt-4.1", "gpt-4.1-mini"]
    };
    assert.equal(provider.id, "openai");
    assert.equal(provider.kind, "openai");
    assert.ok(provider.enabled);
    assert.ok(provider.requiresKey);
    assert.ok(provider.hasKey);
    assert.equal(provider.models.length, 2);
  });

  it("accepts a ChatProvider without baseUrl (optional field)", () => {
    const provider: ChatProvider = {
      id: "eie-local",
      kind: "eie-local" as ProviderKind,
      label: "EIE Local",
      enabled: true,
      requiresKey: false,
      hasKey: false,
      models: ["qwen3-4b-q4-k-m"]
    };
    assert.equal(provider.baseUrl, undefined);
    assert.ok(!provider.requiresKey);
  });
});

describe("api module – Conversation interface contract", () => {
  it("accepts a valid Conversation object", () => {
    const conversation: Conversation = {
      id: "conv-1",
      title: "Test chat",
      providerId: "eie-local",
      model: "qwen3-4b-q4-k-m",
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString()
    };
    assert.equal(conversation.id, "conv-1");
    assert.equal(conversation.providerId, "eie-local");
    assert.ok(conversation.createdAt.length > 0);
  });
});

describe("api module – Message interface contract", () => {
  it("accepts a user message with all required fields", () => {
    const message: Message = {
      id: "msg-1",
      conversationId: "conv-1",
      role: "user",
      content: "Hello, world",
      status: "complete",
      createdAt: new Date().toISOString()
    };
    assert.equal(message.role, "user");
    assert.equal(message.status, "complete");
    assert.equal(message.parentId, undefined);
    assert.equal(message.citations, undefined);
  });

  it("accepts an assistant message with parentId and citations", () => {
    const message: Message = {
      id: "msg-2",
      conversationId: "conv-1",
      role: "assistant",
      content: "Here is the answer.",
      status: "complete",
      parentId: "msg-1",
      createdAt: new Date().toISOString(),
      citations: [{ sourceTitle: "doc.md", content: "relevant chunk", score: 0.9 }]
    };
    assert.equal(message.role, "assistant");
    assert.equal(message.parentId, "msg-1");
    assert.equal(message.citations?.length, 1);
    assert.equal(message.citations?.[0].sourceTitle, "doc.md");
  });

  it("supports streaming status", () => {
    const message: Message = {
      id: "msg-3",
      conversationId: "conv-1",
      role: "assistant",
      content: "",
      status: "streaming",
      createdAt: new Date().toISOString()
    };
    assert.equal(message.status, "streaming");
  });

  it("supports error status", () => {
    const message: Message = {
      id: "msg-err",
      conversationId: "conv-1",
      role: "assistant",
      content: "Network error",
      status: "error",
      createdAt: new Date().toISOString()
    };
    assert.equal(message.status, "error");
  });
});

describe("api module – Preset interface contract", () => {
  it("accepts a valid Preset object", () => {
    const preset: Preset = {
      id: "preset-1",
      name: "Balanced local",
      providerId: "eie-local",
      model: "qwen3-4b-q4-k-m",
      systemPrompt: "You are a helpful assistant.",
      temperature: 0.7,
      topP: 0.9,
      maxTokens: 1024,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString()
    };
    assert.equal(preset.name, "Balanced local");
    assert.equal(preset.providerId, "eie-local");
    assert.equal(preset.temperature, 0.7);
    assert.equal(preset.maxTokens, 1024);
  });
});

describe("api module – ChatPayload interface contract", () => {
  it("accepts a payload with provider_id and base_url fields", () => {
    const payload: ChatPayload = {
      conversation_id: "conv-1",
      provider_id: "openai",
      base_url: "https://api.openai.com/v1",
      model: "gpt-4.1",
      messages: [{ role: "user", content: "Hello" }],
      temperature: 0.7,
      top_p: 0.9,
      max_tokens: 1024
    };
    assert.equal(payload.provider_id, "openai");
    assert.equal(payload.base_url, "https://api.openai.com/v1");
    assert.equal(payload.model, "gpt-4.1");
  });

  it("accepts a payload without provider_id (optional – defaults to eie-local)", () => {
    const payload: ChatPayload = {
      model: "qwen3-4b-q4-k-m",
      messages: [{ role: "user", content: "Hi" }],
      temperature: 0.7,
      top_p: 0.9,
      max_tokens: 512
    };
    assert.equal(payload.provider_id, undefined);
    assert.equal(payload.base_url, undefined);
    assert.equal(payload.conversation_id, undefined);
  });

  it("accepts knowledge_stack_ids and retrieval_options fields", () => {
    const payload: ChatPayload = {
      model: "qwen3-4b-q4-k-m",
      messages: [],
      temperature: 0.7,
      top_p: 0.9,
      max_tokens: 1024,
      knowledge_stack_ids: ["stack-1", "stack-2"],
      retrieval_options: { top_k: 5, semantic_weight: 0.7 }
    };
    assert.deepEqual(payload.knowledge_stack_ids, ["stack-1", "stack-2"]);
    assert.equal(payload.retrieval_options?.top_k, 5);
  });
});

describe("api module – ProviderKind union type", () => {
  it("accepts all valid provider kind values", () => {
    const kinds: ProviderKind[] = [
      "eie-local",
      "openai",
      "anthropic",
      "google",
      "openai-compatible"
    ];
    assert.equal(kinds.length, 5);
    assert.ok(kinds.includes("eie-local"));
    assert.ok(kinds.includes("anthropic"));
    assert.ok(kinds.includes("openai-compatible"));
  });
});
