import type { ChatKnowledgeFields } from "./api";

export function toggleStackSelection(selectedIds: string[], stackId: string): string[] {
  return selectedIds.includes(stackId)
    ? selectedIds.filter((id) => id !== stackId)
    : [...selectedIds, stackId];
}

export function formatSourceStatus(status: string): string {
  switch (status) {
    case "indexed":
      return "Indexed";
    case "extracting":
      return "Extracting";
    case "failed":
      return "Failed";
    case "pending":
      return "Pending";
    default:
      return status ? status[0].toUpperCase() + status.slice(1) : "Unknown";
  }
}

export function buildKnowledgeChatFields(activeStackIds: string[]): ChatKnowledgeFields {
  if (activeStackIds.length === 0) {
    return {};
  }

  return {
    knowledge_stack_ids: activeStackIds,
    retrieval_options: {
      top_k: 6,
      semantic_weight: 0.65
    }
  };
}
