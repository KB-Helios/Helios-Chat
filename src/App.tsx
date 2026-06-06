import {
  Activity,
  AlertTriangle,
  BookOpen,
  CheckCircle2,
  Cpu,
  Database,
  Download,
  FileText,
  FolderOpen,
  HardDrive,
  Layers,
  MessageSquare,
  Play,
  Plus,
  RefreshCw,
  Search,
  Send,
  Settings,
  SlidersHorizontal,
  Square,
  Terminal,
  Trash2,
  Upload,
  Zap
} from "lucide-react";
import { FormEvent, useEffect, useMemo, useState } from "react";
import {
  defaultSettings,
  chatSend,
  engineStart,
  engineStatus,
  engineStop,
  isTauriRuntime,
  knowledgeSearch,
  knowledgeSourceRemove,
  knowledgeSourcesAddFiles,
  knowledgeSourcesAddFolder,
  knowledgeSourcesList,
  knowledgeStackCreate,
  knowledgeStackReindex,
  knowledgeStacksList,
  modelsCatalog,
  modelsDownload,
  modelsImportLocal,
  modelsLoad,
  modelsSetDefault,
  modelsUnload,
  settingsGet,
  settingsUpdate,
  setupBuildEie,
  setupCheckPrereqs,
  type EngineStatus,
  type HeliosSettings,
  type KnowledgeSearchResult,
  type KnowledgeSource,
  type KnowledgeStack,
  type ToolStatus
} from "./lib/api";
import { addUserMessage, appendAssistantToken, attachAssistantCitations, createInitialChatState, finishAssistantMessage, startAssistantMessage, type ChatMessage, type ChatState } from "./lib/chatState";
import { catalogById, formatBytes, recommendedModel, type CatalogModel } from "./lib/catalog";
import { buildKnowledgeChatFields, formatSourceStatus, toggleStackSelection } from "./lib/knowledgeState";

const sampleConversations = [
  { id: "local", title: "Local EIE session", meta: "Qwen3 4B" },
  { id: "setup", title: "Setup notes", meta: "Toolchain" },
  { id: "models", title: "Model testing", meta: "GGUF" }
];

export default function App() {
  const [catalog, setCatalog] = useState<CatalogModel[]>([]);
  const [settings, setSettings] = useState<HeliosSettings>(defaultSettings);
  const [tools, setTools] = useState<ToolStatus[]>([]);
  const [engine, setEngine] = useState<EngineStatus>({ running: false, endpoint: "http://127.0.0.1:8090", detail: "Checking engine..." });
  const [chat, setChat] = useState<ChatState>(() => createInitialChatState("Qwen3 4B"));
  const [prompt, setPrompt] = useState("");
  const [view, setView] = useState<"chat" | "knowledge">("chat");
  const [selectedTab, setSelectedTab] = useState<"models" | "settings">("models");
  const [activity, setActivity] = useState("Idle");
  const [busy, setBusy] = useState(false);
  const [knowledgeStacks, setKnowledgeStacks] = useState<KnowledgeStack[]>([]);
  const [selectedStackId, setSelectedStackId] = useState<string>();
  const [activeStackIds, setActiveStackIds] = useState<string[]>([]);
  const [knowledgeSources, setKnowledgeSources] = useState<KnowledgeSource[]>([]);
  const [knowledgeQuery, setKnowledgeQuery] = useState("");
  const [knowledgeResults, setKnowledgeResults] = useState<KnowledgeSearchResult[]>([]);
  const [newStackName, setNewStackName] = useState("Research Stack");
  const [newStackDescription, setNewStackDescription] = useState("Local files and folders");

  useEffect(() => {
    void refreshAll();
  }, []);

  useEffect(() => {
    if (selectedStackId) {
      void refreshSources(selectedStackId);
    } else {
      setKnowledgeSources([]);
    }
  }, [selectedStackId]);

  const byId = useMemo(() => catalogById(catalog), [catalog]);
  const activeModel = settings.default_model_id ? byId[settings.default_model_id] : recommendedModel(catalog);
  const readyTools = tools.filter((tool) => tool.present).length;
  const requiredToolsReady = tools.filter((tool) => ["git", "cmake", "cl"].includes(tool.name)).every((tool) => tool.present);

  async function refreshAll() {
    const [nextCatalog, nextSettings, nextTools, nextEngine, nextStacks] = await Promise.all([
      modelsCatalog(),
      settingsGet(),
      setupCheckPrereqs(),
      engineStatus(),
      knowledgeStacksList()
    ]);
    setCatalog(nextCatalog);
    setSettings(nextSettings.default_model_id ? nextSettings : { ...nextSettings, default_model_id: recommendedModel(nextCatalog)?.id });
    setTools(nextTools);
    setEngine(nextEngine);
    setChat((state) => ({ ...state, activeModelName: recommendedModel(nextCatalog)?.name ?? "Local model" }));
    setKnowledgeStacks(nextStacks);
    setSelectedStackId((current) => current ?? nextStacks[0]?.id);
  }

  async function refreshKnowledge() {
    const stacks = await knowledgeStacksList();
    setKnowledgeStacks(stacks);
    setSelectedStackId((current) => current && stacks.some((stack) => stack.id === current) ? current : stacks[0]?.id);
  }

  async function refreshSources(stackId = selectedStackId) {
    if (!stackId) {
      setKnowledgeSources([]);
      return;
    }
    setKnowledgeSources(await knowledgeSourcesList(stackId));
  }

  async function handleBuild() {
    setBusy(true);
    setActivity("Preparing EIE build");
    try {
      const result = await setupBuildEie();
      setActivity(`Prepared ${result.backend.toUpperCase()} build`);
    } catch (error) {
      setActivity(error instanceof Error ? error.message : String(error));
    } finally {
      setBusy(false);
    }
  }

  async function handleEngineToggle() {
    setBusy(true);
    try {
      const next = engine.running ? await engineStop() : await engineStart();
      setEngine(next);
      setActivity(next.detail);
    } catch (error) {
      setActivity(error instanceof Error ? error.message : String(error));
    } finally {
      setBusy(false);
    }
  }

  async function handleDefaultModel(model: CatalogModel) {
    const next = await modelsSetDefault(model.id);
    setSettings(next);
    setChat((state) => ({ ...state, activeModelName: model.name }));
    setActivity(`${model.name} is the default`);
  }

  async function handleDownload(model: CatalogModel) {
    setBusy(true);
    setActivity(`Downloading ${model.name}`);
    try {
      await modelsDownload(model.id);
      setActivity(`${model.name} downloaded`);
    } catch (error) {
      setActivity(error instanceof Error ? error.message : String(error));
    } finally {
      setBusy(false);
    }
  }

  async function handleImport() {
    if (!isTauriRuntime()) {
      setActivity("Local GGUF import is available in the desktop runtime");
      return;
    }

    const { open } = await import("@tauri-apps/plugin-dialog");
    const selected = await open({
      multiple: false,
      filters: [{ name: "GGUF model", extensions: ["gguf"] }]
    });
    if (typeof selected !== "string") {
      return;
    }
    const imported = await modelsImportLocal(selected);
    setActivity(`Imported ${imported}`);
  }

  async function handleLoad(model: CatalogModel) {
    setBusy(true);
    setActivity(`Loading ${model.name}`);
    try {
      await modelsLoad(model.id);
      setActivity(`${model.name} loaded`);
    } catch (error) {
      setActivity(error instanceof Error ? error.message : String(error));
    } finally {
      setBusy(false);
    }
  }

  async function handleUnload(model: CatalogModel) {
    setBusy(true);
    setActivity(`Unloading ${model.name}`);
    try {
      await modelsUnload(model.id);
      setActivity(`${model.name} unloaded`);
    } catch (error) {
      setActivity(error instanceof Error ? error.message : String(error));
    } finally {
      setBusy(false);
    }
  }

  async function handleSettingsPatch(patch: Partial<HeliosSettings>) {
    const next = { ...settings, ...patch };
    setSettings(next);
    await settingsUpdate(next);
  }

  async function handleCreateStack() {
    const name = newStackName.trim() || "Untitled Stack";
    const stack = await knowledgeStackCreate(name, newStackDescription.trim());
    setKnowledgeStacks((stacks) => [stack, ...stacks]);
    setSelectedStackId(stack.id);
    setNewStackName("Research Stack");
    setNewStackDescription("Local files and folders");
    setActivity(`Created ${stack.name}`);
  }

  async function handleAddKnowledgeFiles() {
    if (!selectedStackId) {
      setActivity("Create a knowledge stack first");
      return;
    }

    let paths: string[] = ["C:/Helios/docs/local.md"];
    if (isTauriRuntime()) {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({
        multiple: true,
        filters: [
          { name: "Knowledge files", extensions: ["txt", "md", "csv", "json", "jsonl", "pdf", "docx", "rtf", "epub"] }
        ]
      });
      if (!selected) {
        return;
      }
      paths = Array.isArray(selected) ? selected : [selected];
    }

    setBusy(true);
    setActivity("Indexing files");
    try {
      await knowledgeSourcesAddFiles(selectedStackId, paths);
      await refreshKnowledge();
      await refreshSources(selectedStackId);
      setActivity(`Indexed ${paths.length} file${paths.length === 1 ? "" : "s"}`);
    } catch (error) {
      setActivity(error instanceof Error ? error.message : String(error));
    } finally {
      setBusy(false);
    }
  }

  async function handleAddKnowledgeFolder() {
    if (!selectedStackId) {
      setActivity("Create a knowledge stack first");
      return;
    }

    let folder = "C:/Helios/docs";
    if (isTauriRuntime()) {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({ directory: true, multiple: false });
      if (typeof selected !== "string") {
        return;
      }
      folder = selected;
    }

    setBusy(true);
    setActivity("Indexing folder");
    try {
      const sources = await knowledgeSourcesAddFolder(selectedStackId, folder);
      await refreshKnowledge();
      await refreshSources(selectedStackId);
      setActivity(`Indexed ${sources.length} source${sources.length === 1 ? "" : "s"}`);
    } catch (error) {
      setActivity(error instanceof Error ? error.message : String(error));
    } finally {
      setBusy(false);
    }
  }

  async function handleReindexStack() {
    if (!selectedStackId) {
      return;
    }

    setBusy(true);
    setActivity("Reindexing stack");
    try {
      const sources = await knowledgeStackReindex(selectedStackId);
      await refreshKnowledge();
      await refreshSources(selectedStackId);
      setActivity(`Reindexed ${sources.length} source${sources.length === 1 ? "" : "s"}`);
    } catch (error) {
      setActivity(error instanceof Error ? error.message : String(error));
    } finally {
      setBusy(false);
    }
  }

  async function handleRemoveSource(sourceId: string) {
    await knowledgeSourceRemove(sourceId);
    await refreshKnowledge();
    await refreshSources(selectedStackId);
    setActivity("Removed source");
  }

  async function handleKnowledgeSearch() {
    const query = knowledgeQuery.trim();
    if (!query || !selectedStackId) {
      return;
    }
    const results = await knowledgeSearch([selectedStackId], query, { top_k: 6, semantic_weight: 0.65 });
    setKnowledgeResults(results);
    setActivity(`Found ${results.length} result${results.length === 1 ? "" : "s"}`);
  }

  function handleToggleActiveStack(stackId: string) {
    setActiveStackIds((ids) => toggleStackSelection(ids, stackId));
  }

  async function handleSend(event: FormEvent) {
    event.preventDefault();
    const trimmed = prompt.trim();
    if (!trimmed || busy) {
      return;
    }

    setPrompt("");
    setBusy(true);
    setActivity("Generating");
    const assistantId = crypto.randomUUID();
    const userChat = addUserMessage(chat, trimmed);
    let nextChat = startAssistantMessage(userChat, assistantId);
    setChat(nextChat);

    if (isTauriRuntime()) {
      let receivedToken = false;
      const { listen } = await import("@tauri-apps/api/event");
      const unlistenToken = await listen<string>("chat:token", (event) => {
        receivedToken = true;
        nextChat = appendAssistantToken(nextChat, String(event.payload ?? ""));
        setChat(nextChat);
      });
      const unlistenDone = await listen<string>("chat:done", () => {
        setActivity("Idle");
      });

      try {
        const response = await chatSend({
          model: activeModel?.id ?? settings.default_model_id ?? defaultSettings.default_model_id ?? "qwen3-4b-q4-k-m",
          messages: userChat.messages.map((message) => ({ role: message.role, content: message.content })),
          temperature: settings.temperature,
          top_p: settings.top_p,
          max_tokens: settings.max_tokens,
          ...buildKnowledgeChatFields(activeStackIds)
        });
        if (!receivedToken && response.content) {
          nextChat = appendAssistantToken(nextChat, response.content);
          setChat(nextChat);
        }
        nextChat = attachAssistantCitations(nextChat, response.citations.map((citation) => ({
          sourceTitle: citation.source_title,
          content: citation.content,
          score: citation.score
        })));
        setChat(finishAssistantMessage(nextChat));
        setActivity("Idle");
      } catch (error) {
        nextChat = appendAssistantToken(nextChat, error instanceof Error ? error.message : String(error));
        setChat(finishAssistantMessage(nextChat));
        setActivity(error instanceof Error ? error.message : String(error));
      } finally {
        unlistenToken();
        unlistenDone();
        setBusy(false);
      }
      return;
    }

    const tokens = ["EIE ", "is ", "wired ", "as ", "the ", "default ", "local ", "engine. ", "Complete ", "first-run ", "setup ", "to ", "replace ", "this ", "browser ", "preview ", "with ", "real ", "model ", "tokens."];
    for (const token of tokens) {
      await new Promise((resolve) => window.setTimeout(resolve, 45));
      nextChat = appendAssistantToken(nextChat, token);
      setChat(nextChat);
    }
    nextChat = attachAssistantCitations(nextChat, activeStackIds.length ? [{
      sourceTitle: "local.md",
      content: "Helios Knowledge Hub keeps private documents searchable on this machine.",
      score: 0.92
    }] : []);
    setChat(finishAssistantMessage(nextChat));
    setActivity("Idle");
    setBusy(false);
  }

  return (
    <main className="app-shell">
      <aside className="sidebar">
        <div className="brand-row">
          <div className="brand-mark"><Zap size={20} /></div>
          <div>
            <h1>Helios Chat</h1>
            <p>EIE local runtime</p>
          </div>
        </div>

        <div className="engine-strip">
          <div className={engine.running ? "status-dot online" : "status-dot"} />
          <div>
            <strong>{engine.running ? "Engine online" : "Engine offline"}</strong>
            <span>{engine.endpoint}</span>
          </div>
          <button className="icon-button" onClick={handleEngineToggle} disabled={busy} title={engine.running ? "Stop EIE" : "Start EIE"}>
            {engine.running ? <Square size={17} /> : <Play size={17} />}
          </button>
        </div>

        <nav className="conversation-list" aria-label="Conversations">
          <button className={view === "chat" ? "conversation-item active" : "conversation-item"} onClick={() => setView("chat")}>
            <MessageSquare size={16} />
            <span>Chat</span>
            <small>{activeStackIds.length ? `${activeStackIds.length} stack active` : "Local session"}</small>
          </button>
          <button className={view === "knowledge" ? "conversation-item active" : "conversation-item"} onClick={() => setView("knowledge")}>
            <BookOpen size={16} />
            <span>Knowledge Hub</span>
            <small>{knowledgeStacks.length} stack{knowledgeStacks.length === 1 ? "" : "s"}</small>
          </button>
          {view === "chat" ? sampleConversations.map((conversation) => (
            <button className="conversation-item" key={conversation.id}>
              <MessageSquare size={16} />
              <span>{conversation.title}</span>
              <small>{conversation.meta}</small>
            </button>
          )) : null}
        </nav>
      </aside>

      <section className="workbench">
        <header className="topbar">
          <div>
            <span className="eyebrow">{view === "chat" ? "Default model" : "Local knowledge"}</span>
            <h2>{view === "chat" ? activeModel?.name ?? "No model selected" : "Knowledge Hub"}</h2>
          </div>
          <div className="topbar-actions">
            <button className="toolbar-button" onClick={refreshAll} title="Refresh status">
              <RefreshCw size={17} />
              Refresh
            </button>
            {view === "chat" ? <button className="toolbar-button primary" onClick={handleBuild} disabled={busy || !requiredToolsReady} title="Build EIE">
              <Terminal size={17} />
              Build EIE
            </button> : <button className="toolbar-button primary" onClick={handleCreateStack} disabled={busy} title="Create stack">
              <Plus size={17} />
              New Stack
            </button>}
          </div>
        </header>

        {view === "chat" ? <div className="main-grid">
          <section className="chat-panel">
            <div className="message-scroll">
              {chat.messages.length === 0 ? (
                <div className="empty-state">
                  <Activity size={28} />
                  <strong>Ready for a local session</strong>
                  <span>{activeModel?.name ?? "Choose a GGUF model"} through EIE</span>
                </div>
              ) : (
                chat.messages.map((message) => <MessageBubble key={message.id} message={message} />)
              )}
            </div>

            <form className="composer" onSubmit={handleSend}>
              {knowledgeStacks.length ? (
                <div className="active-stack-row">
                  <Layers size={15} />
                  {knowledgeStacks.map((stack) => (
                    <button
                      type="button"
                      className={activeStackIds.includes(stack.id) ? "stack-chip selected" : "stack-chip"}
                      key={stack.id}
                      onClick={() => handleToggleActiveStack(stack.id)}
                    >
                      {stack.name}
                    </button>
                  ))}
                </div>
              ) : null}
              <textarea value={prompt} onChange={(event) => setPrompt(event.target.value)} placeholder="Ask Helios..." rows={3} />
              <button className="send-button" disabled={!prompt.trim() || busy} title="Send message">
                <Send size={18} />
              </button>
            </form>
          </section>

          <aside className="control-panel">
            <section className="setup-band">
              <div className="section-heading">
                <Cpu size={18} />
                <h3>First Run</h3>
                <span>{readyTools}/{tools.length || 4}</span>
              </div>
              <div className="tool-list">
                {tools.map((tool) => (
                  <a className="tool-row" href={tool.install_url || undefined} key={tool.name} target="_blank" rel="noreferrer">
                    {tool.present ? <CheckCircle2 size={16} /> : <AlertTriangle size={16} />}
                    <span>{tool.name}</span>
                    <small>{tool.present ? "Ready" : "Missing"}</small>
                  </a>
                ))}
              </div>
            </section>

            <div className="tabs">
              <button className={selectedTab === "models" ? "selected" : ""} onClick={() => setSelectedTab("models")}>
                <Database size={16} /> Models
              </button>
              <button className={selectedTab === "settings" ? "selected" : ""} onClick={() => setSelectedTab("settings")}>
                <Settings size={16} /> Settings
              </button>
            </div>

            {selectedTab === "models" ? (
              <section className="model-list">
                {catalog.map((model) => (
                  <article className={model.id === settings.default_model_id ? "model-card selected" : "model-card"} key={model.id}>
                    <div>
                      <h3>{model.name}</h3>
                      <p>{model.description}</p>
                    </div>
                    <div className="model-meta">
                      <span><HardDrive size={14} /> {formatBytes(model.sizeBytes)}</span>
                      <span>{model.quantization}</span>
                      <span>{model.minimumVramGb} GB VRAM</span>
                    </div>
                    <div className="model-actions">
                      <button onClick={() => handleDefaultModel(model)}>{model.id === settings.default_model_id ? "Default" : "Set default"}</button>
                      <button onClick={() => handleLoad(model)} disabled={busy || !engine.running}>Load</button>
                      <button onClick={() => handleUnload(model)} disabled={busy || !engine.running}>Unload</button>
                      <button className="icon-button" onClick={() => handleDownload(model)} disabled={busy} title={`Download ${model.name}`}>
                        <Download size={16} />
                      </button>
                    </div>
                  </article>
                ))}
                <button className="import-button" onClick={handleImport}>
                  <Upload size={16} />
                  Import GGUF
                </button>
              </section>
            ) : (
              <section className="settings-list">
                <label>
                  <span><SlidersHorizontal size={15} /> Temperature</span>
                  <input type="range" min="0" max="1.5" step="0.1" value={settings.temperature} onChange={(event) => handleSettingsPatch({ temperature: Number(event.target.value) })} />
                  <strong>{settings.temperature.toFixed(1)}</strong>
                </label>
                <label>
                  <span>Top P</span>
                  <input type="range" min="0.1" max="1" step="0.05" value={settings.top_p} onChange={(event) => handleSettingsPatch({ top_p: Number(event.target.value) })} />
                  <strong>{settings.top_p.toFixed(2)}</strong>
                </label>
                <label>
                  <span>Context</span>
                  <input type="number" min="1024" max="32768" step="1024" value={settings.n_ctx} onChange={(event) => handleSettingsPatch({ n_ctx: Number(event.target.value) })} />
                </label>
                <label>
                  <span>Max tokens</span>
                  <input type="number" min="64" max="8192" step="64" value={settings.max_tokens} onChange={(event) => handleSettingsPatch({ max_tokens: Number(event.target.value) })} />
                </label>
                <label>
                  <span>GPU layers</span>
                  <input type="number" min="0" max="99" value={settings.n_gpu_layers} onChange={(event) => handleSettingsPatch({ n_gpu_layers: Number(event.target.value) })} />
                </label>
                <label>
                  <span>KV key</span>
                  <select value={settings.kv_type_k} onChange={(event) => handleSettingsPatch({ kv_type_k: event.target.value })}>
                    <option value="turbo3">turbo3</option>
                    <option value="turbo4">turbo4</option>
                    <option value="q8_0">q8_0</option>
                    <option value="f16">f16</option>
                  </select>
                </label>
                <label>
                  <span>KV value</span>
                  <select value={settings.kv_type_v} onChange={(event) => handleSettingsPatch({ kv_type_v: event.target.value })}>
                    <option value="turbo3">turbo3</option>
                    <option value="turbo2">turbo2</option>
                    <option value="turbo4">turbo4</option>
                    <option value="q8_0">q8_0</option>
                    <option value="f16">f16</option>
                  </select>
                </label>
              </section>
            )}

            <footer className="activity-line">{activity}</footer>
          </aside>
        </div> : (
          <KnowledgeHub
            activity={activity}
            busy={busy}
            knowledgeQuery={knowledgeQuery}
            newStackDescription={newStackDescription}
            newStackName={newStackName}
            results={knowledgeResults}
            selectedStackId={selectedStackId}
            sources={knowledgeSources}
            stacks={knowledgeStacks}
            onAddFiles={handleAddKnowledgeFiles}
            onAddFolder={handleAddKnowledgeFolder}
            onCreateStack={handleCreateStack}
            onQueryChange={setKnowledgeQuery}
            onReindex={handleReindexStack}
            onRemoveSource={handleRemoveSource}
            onSearch={handleKnowledgeSearch}
            onSelectStack={setSelectedStackId}
            onStackDescriptionChange={setNewStackDescription}
            onStackNameChange={setNewStackName}
            activeStackIds={activeStackIds}
            onToggleActiveStack={handleToggleActiveStack}
          />
        )}
      </section>
    </main>
  );
}

function MessageBubble({ message }: { message: ChatMessage }) {
  return (
    <article className={`message ${message.role}`}>
      <span>{message.role}</span>
      <p>{message.content}{message.streaming ? <i /> : null}</p>
      {message.citations?.length ? (
        <div className="citation-row">
          {message.citations.map((citation, index) => (
            <button className="citation-chip" key={`${citation.sourceTitle}-${index}`} title={citation.content}>
              <FileText size={13} />
              {citation.sourceTitle}
            </button>
          ))}
        </div>
      ) : null}
    </article>
  );
}

function KnowledgeHub({
  activeStackIds,
  activity,
  busy,
  knowledgeQuery,
  newStackDescription,
  newStackName,
  results,
  selectedStackId,
  sources,
  stacks,
  onAddFiles,
  onAddFolder,
  onCreateStack,
  onQueryChange,
  onReindex,
  onRemoveSource,
  onSearch,
  onSelectStack,
  onStackDescriptionChange,
  onStackNameChange,
  onToggleActiveStack
}: {
  activeStackIds: string[];
  activity: string;
  busy: boolean;
  knowledgeQuery: string;
  newStackDescription: string;
  newStackName: string;
  results: KnowledgeSearchResult[];
  selectedStackId?: string;
  sources: KnowledgeSource[];
  stacks: KnowledgeStack[];
  onAddFiles: () => void;
  onAddFolder: () => void;
  onCreateStack: () => void;
  onQueryChange: (value: string) => void;
  onReindex: () => void;
  onRemoveSource: (sourceId: string) => void;
  onSearch: () => void;
  onSelectStack: (stackId: string) => void;
  onStackDescriptionChange: (value: string) => void;
  onStackNameChange: (value: string) => void;
  onToggleActiveStack: (stackId: string) => void;
}) {
  const selectedStack = stacks.find((stack) => stack.id === selectedStackId);

  return (
    <div className="knowledge-layout">
      <aside className="knowledge-sidebar">
        <section className="stack-create">
          <input value={newStackName} onChange={(event) => onStackNameChange(event.target.value)} aria-label="Stack name" />
          <textarea value={newStackDescription} onChange={(event) => onStackDescriptionChange(event.target.value)} aria-label="Stack description" rows={2} />
          <button className="toolbar-button primary" onClick={onCreateStack} disabled={busy}>
            <Plus size={16} />
            Create
          </button>
        </section>

        <section className="stack-list" aria-label="Knowledge stacks">
          {stacks.length === 0 ? (
            <div className="empty-card">
              <BookOpen size={22} />
              <strong>No stacks yet</strong>
              <span>Create one to start indexing local files.</span>
            </div>
          ) : stacks.map((stack) => (
            <article className={stack.id === selectedStackId ? "stack-card selected" : "stack-card"} key={stack.id}>
              <button onClick={() => onSelectStack(stack.id)}>
                <strong>{stack.name}</strong>
                <span>{stack.indexed_source_count}/{stack.source_count} indexed</span>
              </button>
              <button className={activeStackIds.includes(stack.id) ? "mini-toggle selected" : "mini-toggle"} onClick={() => onToggleActiveStack(stack.id)}>
                {activeStackIds.includes(stack.id) ? "Active" : "Use in chat"}
              </button>
            </article>
          ))}
        </section>
      </aside>

      <section className="knowledge-main">
        <div className="knowledge-header">
          <div>
            <span className="eyebrow">Selected stack</span>
            <h3>{selectedStack?.name ?? "No stack selected"}</h3>
            <p>{selectedStack?.description || "Create or select a stack to manage sources."}</p>
          </div>
          <div className="knowledge-actions">
            <button className="toolbar-button" onClick={onAddFiles} disabled={busy || !selectedStackId}>
              <Upload size={16} />
              Files
            </button>
            <button className="toolbar-button" onClick={onAddFolder} disabled={busy || !selectedStackId}>
              <FolderOpen size={16} />
              Folder
            </button>
            <button className="toolbar-button" onClick={onReindex} disabled={busy || !selectedStackId}>
              <RefreshCw size={16} />
              Reindex
            </button>
          </div>
        </div>

        <div className="knowledge-grid">
          <section className="source-panel">
            <div className="section-heading">
              <FileText size={18} />
              <h3>Sources</h3>
              <span>{sources.length}</span>
            </div>
            <div className="source-list">
              {sources.length === 0 ? (
                <div className="empty-card compact">
                  <span>No sources indexed.</span>
                </div>
              ) : sources.map((source) => (
                <article className={`source-row ${source.status}`} key={source.id}>
                  <FileText size={16} />
                  <div>
                    <strong>{source.title}</strong>
                    <span>{source.format.toUpperCase()} - {formatSourceStatus(source.status)}</span>
                    {source.error ? <small>{source.error}</small> : null}
                  </div>
                  <button className="icon-button" onClick={() => onRemoveSource(source.id)} title="Remove source">
                    <Trash2 size={15} />
                  </button>
                </article>
              ))}
            </div>
          </section>

          <section className="search-panel">
            <div className="section-heading">
              <Search size={18} />
              <h3>Search Test</h3>
              <span>{results.length}</span>
            </div>
            <div className="knowledge-search">
              <input value={knowledgeQuery} onChange={(event) => onQueryChange(event.target.value)} placeholder="Search this stack..." />
              <button className="send-button" onClick={onSearch} disabled={!knowledgeQuery.trim() || !selectedStackId}>
                <Search size={17} />
              </button>
            </div>
            <div className="result-list">
              {results.map((result, index) => (
                <article className="result-card" key={result.chunk_id}>
                  <span>[{index + 1}] {result.source_title} - {(result.score * 100).toFixed(0)}%</span>
                  <p>{result.content}</p>
                </article>
              ))}
            </div>
          </section>
        </div>

        <footer className="activity-line">{activity}</footer>
      </section>
    </div>
  );
}
