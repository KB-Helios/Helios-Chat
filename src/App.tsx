import {
  AlertTriangle,
  BookOpen,
  Bot,
  CheckCircle2,
  Cpu,
  Download,
  Edit3,
  FileText,
  FolderOpen,
  KeyRound,
  Layers,
  MessageSquarePlus,
  Play,
  Plus,
  RefreshCw,
  RotateCcw,
  Search,
  Send,
  Settings,
  SlidersHorizontal,
  Square,
  Trash2,
  Upload,
  Zap
} from "lucide-react";
import { FormEvent, useEffect, useMemo, useState } from "react";
import {
  chatSend,
  conversationCreate,
  conversationDelete,
  conversationUpdate,
  conversationsList,
  defaultSettings,
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
  messageAppend,
  messageUpdate,
  messagesList,
  modelsCatalog,
  modelsDownload,
  modelsImportLocal,
  modelsLoad,
  modelsSetDefault,
  modelsUnload,
  presetsList,
  providerKeyDelete,
  providerKeySet,
  providersList,
  settingsGet,
  settingsUpdate,
  setupBuildEie,
  setupCheckPrereqs,
  type ChatProvider,
  type Conversation,
  type EngineStatus,
  type HeliosSettings,
  type KnowledgeSearchResult,
  type KnowledgeSource,
  type KnowledgeStack,
  type Message,
  type Preset,
  type ToolStatus
} from "./lib/api";
import { catalogById, formatBytes, recommendedModel, type CatalogModel } from "./lib/catalog";
import { buildKnowledgeChatFields, formatSourceStatus, toggleStackSelection } from "./lib/knowledgeState";

type PanelTab = "presets" | "providers" | "eie";

export default function App() {
  const [catalog, setCatalog] = useState<CatalogModel[]>([]);
  const [providers, setProviders] = useState<ChatProvider[]>([]);
  const [conversations, setConversations] = useState<Conversation[]>([]);
  const [messages, setMessages] = useState<Message[]>([]);
  const [presets, setPresets] = useState<Preset[]>([]);
  const [settings, setSettings] = useState<HeliosSettings>(defaultSettings);
  const [tools, setTools] = useState<ToolStatus[]>([]);
  const [engine, setEngine] = useState<EngineStatus>({ running: false, endpoint: "http://127.0.0.1:8090", detail: "Checking engine..." });
  const [activeConversationId, setActiveConversationId] = useState<string>();
  const [activeProviderId, setActiveProviderId] = useState("eie-local");
  const [activeModel, setActiveModel] = useState(defaultSettings.default_model_id ?? "qwen3-4b-q4-k-m");
  const [prompt, setPrompt] = useState("");
  const [search, setSearch] = useState("");
  const [apiKeyDraft, setApiKeyDraft] = useState<Record<string, string>>({});
  const [panelTab, setPanelTab] = useState<PanelTab>("presets");
  const [view, setView] = useState<"chat" | "knowledge">("chat");
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
    void refreshConversations(search);
  }, [search]);

  useEffect(() => {
    if (selectedStackId) {
      void refreshSources(selectedStackId);
    } else {
      setKnowledgeSources([]);
    }
  }, [selectedStackId]);

  const byId = useMemo(() => catalogById(catalog), [catalog]);
  const activeProvider = providers.find((provider) => provider.id === activeProviderId) ?? providers[0];
  const activeConversation = conversations.find((conversation) => conversation.id === activeConversationId);
  const localModel = byId[activeModel] ?? recommendedModel(catalog);
  const providerModels = activeProviderId === "eie-local"
    ? catalog.map((model) => model.id)
    : activeProvider?.models ?? [];
  const requiredToolsReady = tools.filter((tool) => ["git", "cmake", "cl"].includes(tool.name)).every((tool) => tool.present);
  const canSend = Boolean(prompt.trim()) && !busy && Boolean(activeProvider) && (activeProvider.enabled || activeProvider.id === "eie-local");

  async function refreshAll() {
    const [nextCatalog, nextSettings, nextTools, nextEngine, nextProviders, nextConversations, nextPresets, nextStacks] = await Promise.all([
      modelsCatalog(),
      settingsGet(),
      setupCheckPrereqs(),
      engineStatus(),
      providersList(),
      conversationsList(),
      presetsList(),
      knowledgeStacksList()
    ]);
    const recommended = recommendedModel(nextCatalog)?.id ?? defaultSettings.default_model_id ?? "qwen3-4b-q4-k-m";
    setCatalog(nextCatalog);
    setSettings(nextSettings.default_model_id ? nextSettings : { ...nextSettings, default_model_id: recommended });
    setTools(nextTools);
    setEngine(nextEngine);
    setProviders(nextProviders);
    setConversations(nextConversations);
    setPresets(nextPresets);
    setKnowledgeStacks(nextStacks);
    setSelectedStackId((current) => current ?? nextStacks[0]?.id);
    setActiveModel(nextSettings.default_model_id ?? recommended);
    if (nextConversations[0]) {
      await selectConversation(nextConversations[0]);
    }
  }

  async function refreshConversations(term = "") {
    setConversations(await conversationsList(term));
  }

  async function refreshKnowledge() {
    const stacks = await knowledgeStacksList();
    setKnowledgeStacks(stacks);
    setSelectedStackId((current) => current && stacks.some((stack) => stack.id === current) ? current : stacks[0]?.id);
  }

  async function refreshSources(stackId = selectedStackId) {
    setKnowledgeSources(stackId ? await knowledgeSourcesList(stackId) : []);
  }

  async function selectConversation(conversation: Conversation) {
    setActiveConversationId(conversation.id);
    setActiveProviderId(conversation.providerId);
    setActiveModel(conversation.model);
    setMessages(await messagesList(conversation.id));
  }

  async function handleNewChat() {
    const conversation = await conversationCreate("New chat", activeProviderId, activeModel);
    setConversations((items) => [conversation, ...items]);
    setActiveConversationId(conversation.id);
    setMessages([]);
    setView("chat");
    setActivity("New chat ready");
  }

  async function handleDeleteConversation(id: string) {
    await conversationDelete(id);
    const remaining = conversations.filter((conversation) => conversation.id !== id);
    setConversations(remaining);
    if (activeConversationId === id) {
      if (remaining[0]) {
        await selectConversation(remaining[0]);
      } else {
        setActiveConversationId(undefined);
        setMessages([]);
      }
    }
  }

  async function ensureConversation(firstPrompt: string) {
    if (activeConversation) {
      if (activeConversation.providerId !== activeProviderId || activeConversation.model !== activeModel) {
        const updated = await conversationUpdate(activeConversation.id, activeConversation.title, activeProviderId, activeModel);
        setConversations((items) => items.map((item) => item.id === updated.id ? updated : item));
      }
      return activeConversation.id;
    }
    const title = firstPrompt.split(/\s+/).slice(0, 6).join(" ") || "New chat";
    const conversation = await conversationCreate(title, activeProviderId, activeModel);
    setConversations((items) => [conversation, ...items]);
    setActiveConversationId(conversation.id);
    return conversation.id;
  }

  async function handleSend(event: FormEvent) {
    event.preventDefault();
    const trimmed = prompt.trim();
    if (!trimmed || !canSend) {
      return;
    }
    setPrompt("");
    setBusy(true);
    setActivity("Generating");

    try {
      const conversationId = await ensureConversation(trimmed);
      const userMessage = await messageAppend(conversationId, "user", trimmed, "complete");
      const draft = await messageAppend(conversationId, "assistant", "", "streaming", userMessage.id);
      const nextMessages = [...messages, userMessage, draft];
      setMessages(nextMessages);
      await runCompletion(conversationId, draft, nextMessages);
      await refreshConversations(search);
    } finally {
      setBusy(false);
    }
  }

  async function runCompletion(conversationId: string, draft: Message, conversationMessages: Message[]) {
    let content = "";
    let receivedToken = false;
    let unlistenToken: (() => void) | undefined;
    let unlistenDone: (() => void) | undefined;

    if (isTauriRuntime()) {
      const { listen } = await import("@tauri-apps/api/event");
      unlistenToken = await listen<string>("chat:token", (event) => {
        receivedToken = true;
        content += String(event.payload ?? "");
        setMessages((items) => items.map((message) => message.id === draft.id ? { ...message, content } : message));
      });
      unlistenDone = await listen<string>("chat:done", () => setActivity("Idle"));
    }

    try {
      const payloadMessages = [
        ...(settings.system_prompt.trim() ? [{ role: "system", content: settings.system_prompt.trim() }] : []),
        ...conversationMessages
          .filter((message) => message.id !== draft.id)
          .map((message) => ({ role: message.role, content: message.content }))
      ];
      const response = await chatSend({
        conversation_id: conversationId,
        provider_id: activeProviderId,
        base_url: activeProvider?.baseUrl,
        model: activeModel,
        messages: payloadMessages,
        temperature: settings.temperature,
        top_p: settings.top_p,
        max_tokens: settings.max_tokens,
        ...buildKnowledgeChatFields(activeStackIds)
      });
      if (!receivedToken) {
        content = response.content;
        setMessages((items) => items.map((message) => message.id === draft.id ? { ...message, content } : message));
      }
      const citations = response.citations.map((citation) => ({
        sourceTitle: citation.source_title,
        content: citation.content,
        score: citation.score
      }));
      const saved = await messageUpdate(draft.id, content, "complete");
      setMessages((items) => items.map((message) => message.id === draft.id ? { ...saved, citations } : message));
      setActivity("Idle");
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      const saved = await messageUpdate(draft.id, message, "error");
      setMessages((items) => items.map((item) => item.id === draft.id ? saved : item));
      setActivity(message);
    } finally {
      unlistenToken?.();
      unlistenDone?.();
    }
  }

  async function handleRegenerate(message: Message) {
    if (!activeConversationId || busy || !message.parentId) {
      return;
    }
    setBusy(true);
    setActivity("Regenerating");
    try {
      const draft = await messageUpdate(message.id, "", "streaming");
      const kept = messages.slice(0, messages.findIndex((item) => item.id === message.id));
      const nextMessages = [...kept, draft];
      setMessages(nextMessages);
      await runCompletion(activeConversationId, draft, nextMessages);
    } finally {
      setBusy(false);
    }
  }

  async function handleEdit(message: Message) {
    const next = window.prompt("Edit message", message.content);
    if (next == null || next.trim() === message.content.trim() || !activeConversationId || busy) {
      return;
    }
    setBusy(true);
    setActivity("Generating");
    try {
      const saved = await messageUpdate(message.id, next.trim(), "complete");
      const index = messages.findIndex((item) => item.id === message.id);
      const kept = [...messages.slice(0, index), saved];
      const draft = await messageAppend(activeConversationId, "assistant", "", "streaming", saved.id);
      const nextMessages = [...kept, draft];
      setMessages(nextMessages);
      await runCompletion(activeConversationId, draft, nextMessages);
      await refreshConversations(search);
    } finally {
      setBusy(false);
    }
  }

  async function handleProviderKey(provider: ChatProvider) {
    const key = apiKeyDraft[provider.id]?.trim();
    if (!key) {
      return;
    }
    const next = await providerKeySet(provider.id, key);
    setProviders(next);
    setApiKeyDraft((drafts) => ({ ...drafts, [provider.id]: "" }));
    setActivity(`${provider.label} key saved locally`);
  }

  async function handleApplyPreset(preset: Preset) {
    setActiveProviderId(preset.providerId);
    setActiveModel(preset.model);
    await handleSettingsPatch({
      system_prompt: preset.systemPrompt,
      temperature: preset.temperature,
      top_p: preset.topP,
      max_tokens: preset.maxTokens
    });
    setActivity(`${preset.name} applied`);
  }

  async function handleSettingsPatch(patch: Partial<HeliosSettings>) {
    const next = { ...settings, ...patch };
    setSettings(next);
    await settingsUpdate(next);
  }

  async function handleDefaultModel(model: CatalogModel) {
    const next = await modelsSetDefault(model.id);
    setSettings(next);
    setActiveModel(model.id);
    setActivity(`${model.name} is the EIE default`);
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

  async function handleImport() {
    if (!isTauriRuntime()) {
      setActivity("Local GGUF import is available in the desktop runtime");
      return;
    }
    const { open } = await import("@tauri-apps/plugin-dialog");
    const selected = await open({ multiple: false, filters: [{ name: "GGUF model", extensions: ["gguf"] }] });
    if (typeof selected === "string") {
      const imported = await modelsImportLocal(selected);
      setActivity(`Imported ${imported}`);
    }
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
        filters: [{ name: "Knowledge files", extensions: ["txt", "md", "csv", "json", "jsonl", "pdf", "docx", "rtf", "epub"] }]
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

  return (
    <main className="app-shell">
      <aside className="conversation-rail">
        <div className="brand-row">
          <div className="brand-mark"><Zap size={19} /></div>
          <div>
            <h1>Helios Chat</h1>
            <p>EIE-first workspace</p>
          </div>
        </div>

        <button className="new-chat-button" onClick={handleNewChat}>
          <MessageSquarePlus size={17} />
          New chat
        </button>

        <div className="view-switch">
          <button className={view === "chat" ? "selected" : ""} onClick={() => setView("chat")}><Bot size={16} /> Chat</button>
          <button className={view === "knowledge" ? "selected" : ""} onClick={() => setView("knowledge")}><BookOpen size={16} /> Knowledge</button>
        </div>

        {view === "chat" ? (
          <>
            <label className="search-box">
              <Search size={15} />
              <input value={search} onChange={(event) => setSearch(event.target.value)} placeholder="Search chats" />
            </label>
            <nav className="conversation-list" aria-label="Conversations">
              {conversations.map((conversation) => (
                <button className={conversation.id === activeConversationId ? "conversation-item active" : "conversation-item"} key={conversation.id} onClick={() => selectConversation(conversation)}>
                  <span>{conversation.title}</span>
                  <small>{conversation.providerId} / {conversation.model}</small>
                  <Trash2 size={15} onClick={(event) => { event.stopPropagation(); void handleDeleteConversation(conversation.id); }} />
                </button>
              ))}
            </nav>
          </>
        ) : (
          <nav className="conversation-list" aria-label="Knowledge stacks">
            {knowledgeStacks.map((stack) => (
              <button className={stack.id === selectedStackId ? "conversation-item active" : "conversation-item"} key={stack.id} onClick={() => setSelectedStackId(stack.id)}>
                <span>{stack.name}</span>
                <small>{stack.indexed_source_count}/{stack.source_count} indexed</small>
              </button>
            ))}
          </nav>
        )}
      </aside>

      {view === "chat" ? (
        <section className="chat-workbench">
          <header className="chat-topbar">
            <div className="selector-group">
              <label>
                Provider
                <select value={activeProviderId} onChange={(event) => {
                  const provider = providers.find((item) => item.id === event.target.value);
                  setActiveProviderId(event.target.value);
                  setActiveModel(provider?.models[0] ?? activeModel);
                }}>
                  {providers.map((provider) => (
                    <option key={provider.id} value={provider.id}>
                      {provider.label}{provider.id === "eie-local" ? " (default)" : provider.enabled ? "" : " (key needed)"}
                    </option>
                  ))}
                </select>
              </label>
              <label>
                Model
                <select value={activeModel} onChange={(event) => setActiveModel(event.target.value)}>
                  {providerModels.map((model) => (
                    <option key={model} value={model}>{byId[model]?.name ?? model}</option>
                  ))}
                </select>
              </label>
            </div>
            <div className="engine-pill">
              <span className={engine.running ? "status-dot online" : "status-dot"} />
              <strong>{engine.running ? "EIE online" : "EIE offline"}</strong>
              <button className="icon-button" onClick={handleEngineToggle} disabled={busy} title={engine.running ? "Stop EIE" : "Start EIE"}>
                {engine.running ? <Square size={16} /> : <Play size={16} />}
              </button>
            </div>
          </header>

          <div className="message-scroll">
            {messages.length === 0 ? (
              <div className="empty-state">
                <Bot size={30} />
                <strong>{localModel?.name ?? activeModel}</strong>
                <span>{activeProvider?.label ?? "EIE Local"} is selected for this chat.</span>
              </div>
            ) : (
              messages.map((message) => (
                <article className={`message ${message.role} ${message.status}`} key={message.id}>
                  <div className="message-meta">
                    <span>{message.role}</span>
                    <div>
                      {message.role === "user" ? <button className="mini-icon" onClick={() => handleEdit(message)} title="Edit message"><Edit3 size={14} /></button> : null}
                      {message.role === "assistant" ? <button className="mini-icon" onClick={() => handleRegenerate(message)} title="Regenerate"><RotateCcw size={14} /></button> : null}
                    </div>
                  </div>
                  <p>{message.content}{message.status === "streaming" ? <i /> : null}</p>
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
              ))
            )}
          </div>

          <form className="composer" onSubmit={handleSend}>
            {!activeProvider?.enabled && activeProvider?.requiresKey ? (
              <div className="composer-warning"><AlertTriangle size={15} /> Add a local API key for {activeProvider.label} before sending.</div>
            ) : null}
            {knowledgeStacks.length ? (
              <div className="active-stack-row">
                <Layers size={15} />
                {knowledgeStacks.map((stack) => (
                  <button type="button" className={activeStackIds.includes(stack.id) ? "stack-chip selected" : "stack-chip"} key={stack.id} onClick={() => handleToggleActiveStack(stack.id)}>
                    {stack.name}
                  </button>
                ))}
              </div>
            ) : null}
            <textarea value={prompt} onChange={(event) => setPrompt(event.target.value)} placeholder="Ask Helios..." rows={3} />
            <button className="send-button" disabled={!canSend} title="Send message"><Send size={18} /></button>
          </form>
        </section>
      ) : (
        <KnowledgeHub
          activeStackIds={activeStackIds}
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
          onToggleActiveStack={handleToggleActiveStack}
        />
      )}

      <aside className="control-drawer">
        <div className="drawer-tabs">
          <button className={panelTab === "presets" ? "selected" : ""} onClick={() => setPanelTab("presets")}><SlidersHorizontal size={16} /> Presets</button>
          <button className={panelTab === "providers" ? "selected" : ""} onClick={() => setPanelTab("providers")}><KeyRound size={16} /> Keys</button>
          <button className={panelTab === "eie" ? "selected" : ""} onClick={() => setPanelTab("eie")}><Cpu size={16} /> EIE</button>
        </div>

        {panelTab === "presets" ? (
          <section className="drawer-section">
            {presets.map((preset) => (
              <article className="preset-card" key={preset.id}>
                <h3>{preset.name}</h3>
                <p>{preset.providerId} / {preset.model}</p>
                <button onClick={() => handleApplyPreset(preset)}>Apply</button>
              </article>
            ))}
            <div className="settings-list">
              <label><span>System</span><textarea value={settings.system_prompt} onChange={(event) => setSettings((prev) => ({ ...prev, system_prompt: event.target.value }))} onBlur={(event) => handleSettingsPatch({ system_prompt: event.target.value })} rows={3} /></label>
              <label><span>Temperature</span><input type="range" min="0" max="1.5" step="0.1" value={settings.temperature} onChange={(event) => setSettings((prev) => ({ ...prev, temperature: Number(event.target.value) }))} onBlur={(event) => handleSettingsPatch({ temperature: Number(event.target.value) })} onMouseUp={(event) => handleSettingsPatch({ temperature: Number(event.currentTarget.value) })} /><strong>{settings.temperature.toFixed(1)}</strong></label>
              <label><span>Top P</span><input type="range" min="0.1" max="1" step="0.05" value={settings.top_p} onChange={(event) => setSettings((prev) => ({ ...prev, top_p: Number(event.target.value) }))} onBlur={(event) => handleSettingsPatch({ top_p: Number(event.target.value) })} onMouseUp={(event) => handleSettingsPatch({ top_p: Number(event.currentTarget.value) })} /><strong>{settings.top_p.toFixed(2)}</strong></label>
              <label><span>Max tokens</span><input type="number" min="64" max="8192" step="64" value={settings.max_tokens} onChange={(event) => setSettings((prev) => ({ ...prev, max_tokens: Number(event.target.value) }))} onBlur={(event) => handleSettingsPatch({ max_tokens: Number(event.target.value) })} /></label>
            </div>
          </section>
        ) : null}

        {panelTab === "providers" ? (
          <section className="drawer-section">
            {providers.map((provider) => (
              <article className="provider-card" key={provider.id}>
                <div>
                  <h3>{provider.label}</h3>
                  <p>{provider.id === "eie-local" ? "Default local engine" : provider.hasKey ? "Key saved locally" : "No key saved"}</p>
                </div>
                {provider.hasKey || provider.id === "eie-local" ? <CheckCircle2 size={17} /> : <AlertTriangle size={17} />}
                {provider.requiresKey ? (
                  <>
                    <input type="password" value={apiKeyDraft[provider.id] ?? ""} onChange={(event) => setApiKeyDraft((drafts) => ({ ...drafts, [provider.id]: event.target.value }))} placeholder="API key" />
                    <div className="split-actions">
                      <button onClick={() => handleProviderKey(provider)}>Save</button>
                      <button onClick={async () => setProviders(await providerKeyDelete(provider.id))}>Clear</button>
                    </div>
                  </>
                ) : null}
              </article>
            ))}
          </section>
        ) : null}

        {panelTab === "eie" ? (
          <section className="drawer-section">
            <div className="engine-card">
              <strong>{engine.detail}</strong>
              <span>{engine.endpoint}</span>
              <div className="split-actions">
                <button onClick={refreshAll}><RefreshCw size={15} /> Refresh</button>
                <button onClick={handleBuild} disabled={busy || !requiredToolsReady}><Settings size={15} /> Build EIE</button>
              </div>
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
            <div className="model-list">
              {catalog.map((model) => (
                <article className={model.id === settings.default_model_id ? "model-card selected" : "model-card"} key={model.id}>
                  <h3>{model.name}</h3>
                  <p>{model.description}</p>
                  <div className="model-meta">
                    <span>{formatBytes(model.sizeBytes)}</span>
                    <span>{model.quantization}</span>
                    <span>{model.minimumVramGb} GB VRAM</span>
                  </div>
                  <div className="split-actions">
                    <button onClick={() => handleDefaultModel(model)}>{model.id === settings.default_model_id ? "Default" : "Default"}</button>
                    <button onClick={() => handleLoad(model)} disabled={busy || !engine.running}>Load</button>
                    <button onClick={() => handleUnload(model)} disabled={busy || !engine.running}>Unload</button>
                    <button className="icon-button" onClick={() => handleDownload(model)} disabled={busy} title={`Download ${model.name}`}><Download size={15} /></button>
                  </div>
                </article>
              ))}
              <button className="import-button" onClick={handleImport}><Upload size={16} /> Import GGUF</button>
            </div>
          </section>
        ) : null}
        <footer className="activity-line">{activity}</footer>
      </aside>
    </main>
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
    <section className="knowledge-layout">
      <aside className="knowledge-sidebar">
        <section className="stack-create">
          <input value={newStackName} onChange={(event) => onStackNameChange(event.target.value)} aria-label="Stack name" />
          <textarea value={newStackDescription} onChange={(event) => onStackDescriptionChange(event.target.value)} aria-label="Stack description" rows={2} />
          <button className="new-chat-button" onClick={onCreateStack} disabled={busy}>
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
            <button className="split-action-button" onClick={onAddFiles} disabled={busy || !selectedStackId}><Upload size={16} /> Files</button>
            <button className="split-action-button" onClick={onAddFolder} disabled={busy || !selectedStackId}><FolderOpen size={16} /> Folder</button>
            <button className="split-action-button" onClick={onReindex} disabled={busy || !selectedStackId}><RefreshCw size={16} /> Reindex</button>
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
                <div className="empty-card compact"><span>No sources indexed.</span></div>
              ) : sources.map((source) => (
                <article className={`source-row ${source.status}`} key={source.id}>
                  <FileText size={16} />
                  <div>
                    <strong>{source.title}</strong>
                    <span>{source.format.toUpperCase()} - {formatSourceStatus(source.status)}</span>
                    {source.error ? <small>{source.error}</small> : null}
                  </div>
                  <button className="icon-button" onClick={() => onRemoveSource(source.id)} title="Remove source"><Trash2 size={15} /></button>
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
              <button className="send-button" onClick={onSearch} disabled={!knowledgeQuery.trim() || !selectedStackId}><Search size={17} /></button>
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
    </section>
  );
}
