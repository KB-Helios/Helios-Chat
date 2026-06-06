import {
  Activity,
  AlertTriangle,
  CheckCircle2,
  Cpu,
  Database,
  Download,
  HardDrive,
  MessageSquare,
  Play,
  RefreshCw,
  Send,
  Settings,
  SlidersHorizontal,
  Square,
  Terminal,
  Upload,
  Zap
} from "lucide-react";
import { FormEvent, useEffect, useMemo, useState } from "react";
import {
  defaultSettings,
  engineStart,
  engineStatus,
  engineStop,
  isTauriRuntime,
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
  type ToolStatus
} from "./lib/api";
import { addUserMessage, appendAssistantToken, createInitialChatState, finishAssistantMessage, startAssistantMessage, type ChatMessage, type ChatState } from "./lib/chatState";
import { catalogById, formatBytes, recommendedModel, type CatalogModel } from "./lib/catalog";

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
  const [selectedTab, setSelectedTab] = useState<"models" | "settings">("models");
  const [activity, setActivity] = useState("Idle");
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    void refreshAll();
  }, []);

  const byId = useMemo(() => catalogById(catalog), [catalog]);
  const activeModel = settings.default_model_id ? byId[settings.default_model_id] : recommendedModel(catalog);
  const readyTools = tools.filter((tool) => tool.present).length;
  const requiredToolsReady = tools.filter((tool) => ["git", "cmake", "cl"].includes(tool.name)).every((tool) => tool.present);

  async function refreshAll() {
    const [nextCatalog, nextSettings, nextTools, nextEngine] = await Promise.all([
      modelsCatalog(),
      settingsGet(),
      setupCheckPrereqs(),
      engineStatus()
    ]);
    setCatalog(nextCatalog);
    setSettings(nextSettings.default_model_id ? nextSettings : { ...nextSettings, default_model_id: recommendedModel(nextCatalog)?.id });
    setTools(nextTools);
    setEngine(nextEngine);
    setChat((state) => ({ ...state, activeModelName: recommendedModel(nextCatalog)?.name ?? "Local model" }));
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
    let nextChat = startAssistantMessage(addUserMessage(chat, trimmed), assistantId);
    setChat(nextChat);

    const tokens = isTauriRuntime()
      ? ["EIE streaming is active from the desktop runtime."]
      : ["EIE ", "is ", "wired ", "as ", "the ", "default ", "local ", "engine. ", "Complete ", "first-run ", "setup ", "to ", "replace ", "this ", "browser ", "preview ", "with ", "real ", "model ", "tokens."];

    for (const token of tokens) {
      await new Promise((resolve) => window.setTimeout(resolve, 45));
      nextChat = appendAssistantToken(nextChat, token);
      setChat(nextChat);
    }
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
          {sampleConversations.map((conversation) => (
            <button className="conversation-item active" key={conversation.id}>
              <MessageSquare size={16} />
              <span>{conversation.title}</span>
              <small>{conversation.meta}</small>
            </button>
          ))}
        </nav>
      </aside>

      <section className="workbench">
        <header className="topbar">
          <div>
            <span className="eyebrow">Default model</span>
            <h2>{activeModel?.name ?? "No model selected"}</h2>
          </div>
          <div className="topbar-actions">
            <button className="toolbar-button" onClick={refreshAll} title="Refresh status">
              <RefreshCw size={17} />
              Refresh
            </button>
            <button className="toolbar-button primary" onClick={handleBuild} disabled={busy || !requiredToolsReady} title="Build EIE">
              <Terminal size={17} />
              Build EIE
            </button>
          </div>
        </header>

        <div className="main-grid">
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
        </div>
      </section>
    </main>
  );
}

function MessageBubble({ message }: { message: ChatMessage }) {
  return (
    <article className={`message ${message.role}`}>
      <span>{message.role}</span>
      <p>{message.content}{message.streaming ? <i /> : null}</p>
    </article>
  );
}
