export type ChatRole = "system" | "user" | "assistant";

export interface ChatMessage {
  id: string;
  role: ChatRole;
  content: string;
  streaming?: boolean;
  createdAt: string;
}

export interface ChatState {
  activeModelName: string;
  activeAssistantId?: string;
  messages: ChatMessage[];
}

export function createInitialChatState(activeModelName: string): ChatState {
  return {
    activeModelName,
    messages: []
  };
}

export function startAssistantMessage(state: ChatState, id: string): ChatState {
  const draft: ChatMessage = {
    id,
    role: "assistant",
    content: "",
    streaming: true,
    createdAt: new Date().toISOString()
  };

  return {
    ...state,
    activeAssistantId: id,
    messages: [...state.messages, draft]
  };
}

export function appendAssistantToken(state: ChatState, token: string): ChatState {
  if (!state.activeAssistantId) {
    return state;
  }

  return {
    ...state,
    messages: state.messages.map((message) =>
      message.id === state.activeAssistantId
        ? { ...message, content: message.content + token }
        : message
    )
  };
}

export function finishAssistantMessage(state: ChatState): ChatState {
  if (!state.activeAssistantId) {
    return state;
  }

  return {
    ...state,
    activeAssistantId: undefined,
    messages: state.messages.map((message) =>
      message.id === state.activeAssistantId
        ? { ...message, streaming: false }
        : message
    )
  };
}

export function addUserMessage(state: ChatState, content: string): ChatState {
  return {
    ...state,
    messages: [
      ...state.messages,
      {
        id: crypto.randomUUID(),
        role: "user",
        content,
        createdAt: new Date().toISOString()
      }
    ]
  };
}
