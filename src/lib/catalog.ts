export interface CatalogModel {
  id: string;
  name: string;
  description: string;
  hfRepo: string;
  hfFile: string;
  downloadUrl: string;
  sizeBytes: number;
  sha256?: string;
  quantization: string;
  parameters: string;
  recommended: boolean;
  minimumVramGb: number;
  tags: string[];
}

export const defaultCatalog: CatalogModel[] = [
  {
    id: "qwen3-4b-q4-k-m",
    name: "Qwen3 4B Q4_K_M",
    description: "Balanced local default for first-run chat on CPU or modest NVIDIA GPUs.",
    hfRepo: "ggml-org/Qwen3-4B-GGUF",
    hfFile: "Qwen3-4B-Q4_K_M.gguf",
    downloadUrl: "https://huggingface.co/ggml-org/Qwen3-4B-GGUF/resolve/main/Qwen3-4B-Q4_K_M.gguf",
    sizeBytes: 2_500_000_000,
    sha256: "ab27b9bfa375a178d6cba48f3ad892b94b7739659dcc7aae8058ce0ffed6b328",
    quantization: "Q4_K_M",
    parameters: "4B",
    recommended: true,
    minimumVramGb: 4,
    tags: ["balanced", "chat", "gguf"]
  },
  {
    id: "qwen3-8b-q4-k-m",
    name: "Qwen3 8B Q4_K_M",
    description: "Higher quality option for systems with more memory and GPU headroom.",
    hfRepo: "prithivMLmods/Qwen3-8B-GGUF",
    hfFile: "Qwen3_8B.Q4_K_M.gguf",
    downloadUrl: "https://huggingface.co/prithivMLmods/Qwen3-8B-GGUF/resolve/main/Qwen3_8B.Q4_K_M.gguf",
    sizeBytes: 5_030_000_000,
    quantization: "Q4_K_M",
    parameters: "8B",
    recommended: false,
    minimumVramGb: 8,
    tags: ["quality", "chat", "gguf"]
  },
  {
    id: "qwen3-1-7b-q4-k-m",
    name: "Qwen3 1.7B Q4_K_M",
    description: "Small compatibility model for quick CPU fallback checks.",
    hfRepo: "SandLogicTechnologies/Qwen3-GGUF",
    hfFile: "Qwen_Qwen3-1.7B-Q4_K_M.gguf",
    downloadUrl: "https://huggingface.co/SandLogicTechnologies/Qwen3-GGUF/resolve/main/Qwen_Qwen3-1.7B-Q4_K_M.gguf",
    sizeBytes: 1_100_000_000,
    quantization: "Q4_K_M",
    parameters: "1.7B",
    recommended: false,
    minimumVramGb: 2,
    tags: ["small", "cpu", "gguf"]
  }
];

export function recommendedModel(catalog: CatalogModel[]): CatalogModel | undefined {
  return catalog.find((model) => model.recommended) ?? catalog[0];
}

export function catalogById(catalog: CatalogModel[]): Record<string, CatalogModel> {
  return Object.fromEntries(catalog.map((model) => [model.id, model]));
}

export function formatBytes(bytes: number): string {
  if (bytes >= 1_000_000_000) {
    return `${Number((bytes / 1_000_000_000).toFixed(1))} GB`;
  }
  if (bytes >= 1_000_000) {
    return `${Math.round(bytes / 1_000_000)} MB`;
  }
  return `${bytes} B`;
}
