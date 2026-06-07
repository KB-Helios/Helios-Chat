export type ChatRole = "system" | "user" | "assistant";

export interface ChatMessage {
  id: string;
  role: ChatRole;
  content: string;
  status?: "streaming" | "complete" | "error";
  parentId?: string;
  streaming?: boolean;
  createdAt: string;
  citations?: ChatCitation[];
}

export interface ChatState {
  activeModelName: string;
  activeAssistantId?: string;
  messages: ChatMessage[];
}

export interface ChatCitation {
  sourceTitle: string;
  content: string;
  score: number;
}

export function createInitialChatState(activeModelName: string): ChatState {
  return {
    activeModelName,
    messages: []
  };
}

export function startAssistantMessage(state: ChatState, id: string, parentId?: string): ChatState {
  const draft: ChatMessage = {
    id,
    role: "assistant",
    content: "",
    status: "streaming",
    parentId,
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

export function attachAssistantCitations(state: ChatState, citations: ChatCitation[]): ChatState {
  if (!state.activeAssistantId || citations.length === 0) {
    return state;
  }

  return {
    ...state,
    messages: state.messages.map((message) =>
      message.id === state.activeAssistantId
        ? { ...message, citations }
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
        ? { ...message, streaming: false, status: "complete" }
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
        status: "complete",
        createdAt: new Date().toISOString()
      }
    ]
  };
}

export function editMessage(state: ChatState, id: string, content: string): ChatState {
  const index = state.messages.findIndex((message) => message.id === id);
  if (index === -1) {
    return state;
  }

  return {
    ...state,
    activeAssistantId: undefined,
    messages: state.messages.slice(0, index + 1).map((message) =>
      message.id === id
        ? { ...message, content, status: "complete", streaming: false }
        : message
    )
  };
}
