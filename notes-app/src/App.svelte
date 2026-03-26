<script lang="ts">
  import { onMount } from "svelte";
  import { open, save, confirm } from "@tauri-apps/plugin-dialog";
  import { readTextFile, writeTextFile } from "@tauri-apps/plugin-fs";
  import { appState } from "./lib/state.svelte";
  import * as api from "./lib/api";
  import { LSPClient } from "@codemirror/lsp-client";
  import { TauriLspTransport } from "./lib/lspTransport";
  import Toolbar from "./lib/Toolbar.svelte";
  import Sidebar from "./lib/Sidebar.svelte";
  import Editor from "./lib/Editor.svelte";
  import Preview from "./lib/Preview.svelte";
  import SearchModal from "./lib/SearchModal.svelte";
  import CreateNoteModal from "./lib/CreateNoteModal.svelte";
  import RenameModal from "./lib/RenameModal.svelte";
  import ActionsMenu from "./lib/ActionsMenu.svelte";
  import GraphView from "./lib/GraphView.svelte";
  import WelcomeScreen from "./lib/WelcomeScreen.svelte";

  let actionsMenuOpen = $state(false);
  let error = $state("");
  let graphData = $state<{ nodes: any[]; edges: any[] } | null>(null);

  // --- LSP ---

  let lspTransport: TauriLspTransport | null = null;
  let lspClient = $state<LSPClient | null>(null);

  async function startLsp(root: string) {
    try {
      const transport = new TauriLspTransport();
      await transport.start();
      const client = new LSPClient({ rootUri: `file://${root}` });
      client.connect(transport);
      await client.initializing;
      lspTransport = transport;
      lspClient = client;
      console.log("[typos] LSP connected");
    } catch (e) {
      console.warn("[typos] LSP start failed:", e);
      lspClient = null;
      lspTransport = null;
    }
  }

  async function stopLsp() {
    if (lspClient) {
      lspClient.disconnect();
      lspClient = null;
    }
    if (lspTransport) {
      await lspTransport.stop();
      lspTransport = null;
    }
  }

  // --- Vault ---

  async function handleOpenVault() {
    const selected = await open({ directory: true });
    if (!selected) return;
    await openVaultByPath(selected);
  }

  async function handleCreateVault() {
    const selected = await open({ directory: true, title: "Choose where to create your vault" });
    if (!selected) return;
    try {
      const info = await api.initVault(selected);
      appState.vault = info;
      appState.vaultTypes = info.types;
      appState.lastVault = info.root;
      await refreshNotes();
      startLsp(info.root);
      // Open the welcome note in editor
      const welcome = appState.notes.find((n) => n.id === "welcome");
      if (welcome) {
        await handleOpenNote(welcome.id);
      } else {
        await openVaultTyp();
      }
    } catch (e) {
      error = `Failed to create vault: ${e}`;
    }
  }

  async function handleCloseVault() {
    await stopLsp();
    appState.vault = null;
    appState.notes = [];
    appState.vaultTypes = [];
    appState.resetEditor();
    graphData = null;
    error = "";
  }

  async function openVaultByPath(path: string) {
    try {
      const info = await api.openVault(path);
      appState.vault = info;
      appState.vaultTypes = info.types;
      appState.lastVault = info.root;
      await refreshNotes();
      startLsp(info.root);
      await openVaultTyp();
    } catch (e) {
      // If a recent vault no longer exists, remove it silently
      appState.removeRecentVault(path);
      error = `Failed to open vault: ${e}`;
    }
  }

  async function refreshNotes() {
    try {
      appState.notes = await api.listNotes();
    } catch (e) {
      error = `Failed to list notes: ${e}`;
    }
  }

  async function openVaultTyp() {
    if (!appState.vault) return;
    if (appState.isDirty) {
      const discard = await confirm("You have unsaved changes. Discard?", { title: "Unsaved Changes", kind: "warning" });
      if (!discard) return;
    }
    try {
      const vaultTypPath = appState.vault.root + "/vault.typ";
      const content = await readTextFile(vaultTypPath);
      appState.resetEditor();
      graphData = null;
      appState.currentNoteId = "__vault__";
      appState.isVaultTyp = true;
      appState.currentContent = content;
      appState.originalContent = content;
      appState.previewOpen = false;
    } catch (e) {
      error = `Failed to open vault.typ: ${e}`;
    }
  }

  // --- Notes ---

  async function handleOpenNote(id: string) {
    if (appState.isDirty) {
      const discard = await confirm("You have unsaved changes. Discard?", { title: "Unsaved Changes", kind: "warning" });
      if (!discard) return;
    }
    try {
      const content = await api.readNote(id);
      appState.resetEditor();
      graphData = null;
      appState.currentNoteId = id;
      appState.currentContent = content;
      appState.originalContent = content;
      appState.previewOpen = true;
      // Compile for preview
      await handleCompile(id);
    } catch (e) {
      error = `Failed to open note: ${e}`;
    }
  }

  async function handleSave() {
    if (!appState.currentNoteId || !appState.isDirty) return;
    try {
      if (appState.isVaultTyp) {
        const vaultTypPath = appState.vault!.root + "/vault.typ";
        await writeTextFile(vaultTypPath, appState.currentContent);
        appState.markSaved();
        appState.vaultTypes = await api.getVaultTypes();
        await api.reindex();
        await refreshNotes();
        return;
      }
      await api.saveNote(appState.currentNoteId, appState.currentContent);
      appState.markSaved();
      await refreshNotes();
      await handleCompile(appState.currentNoteId);
    } catch (e) {
      error = `Failed to save: ${e}`;
    }
  }

  async function handleCompile(id: string) {
    if (appState.isVaultTyp) return;
    try {
      appState.previewLoading = true;
      const result = await api.compileNote(id, appState.previewFormat);
      appState.previewHtml = result;
    } catch (e) {
      appState.previewHtml = `<pre style="color:red;padding:16px">${e}</pre>`;
    } finally {
      appState.previewLoading = false;
    }
  }

  // Recompile when preview format changes
  let prevFormat = $state(appState.previewFormat);
  $effect(() => {
    const fmt = appState.previewFormat;
    if (fmt !== prevFormat) {
      prevFormat = fmt;
      if (appState.currentNoteId && !appState.isVaultTyp) {
        handleCompile(appState.currentNoteId);
      }
    }
  });

  async function handleDelete() {
    if (!appState.currentNoteId || appState.isVaultTyp) return;
    const ok = await confirm(`Delete "${appState.currentNoteId}"?`, { title: "Delete Note", kind: "warning" });
    if (!ok) return;
    try {
      await api.deleteNote(appState.currentNoteId);
      await api.reindex();
      appState.resetEditor();
      await refreshNotes();
    } catch (e) {
      error = `Failed to delete: ${e}`;
    }
  }

  async function handleRename(newId: string) {
    if (!appState.currentNoteId || appState.isVaultTyp) return;
    try {
      await api.renameNote(appState.currentNoteId, newId);
      await api.reindex();
      appState.currentNoteId = newId;
      await refreshNotes();
      appState.renameModalOpen = false;
    } catch (e) {
      error = `Failed to rename: ${e}`;
    }
  }

  async function handleExportPdf() {
    if (!appState.currentNoteId || appState.isVaultTyp) return;
    const showMeta = await confirm("Include properties (metadata) in the PDF?", {
      title: "Export PDF",
      kind: "info",
      okLabel: "Include",
      cancelLabel: "Without",
    });
    const output = await save({
      defaultPath: `${appState.currentNoteId}.pdf`,
      filters: [{ name: "PDF", extensions: ["pdf"] }],
    });
    if (!output) return;
    try {
      await api.compileNotePdf(appState.currentNoteId, output, showMeta);
    } catch (e) {
      error = `Failed to export PDF: ${e}`;
    }
  }

  async function handleCreateNote(title: string, noteType: string) {
    try {
      const meta = await api.createNote(title, noteType);
      await refreshNotes();
      appState.createModalOpen = false;
      await handleOpenNote(meta.id);
    } catch (e) {
      error = `Failed to create note: ${e}`;
    }
  }

  async function handleTodayJournal() {
    const now = new Date();
    const today = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}-${String(now.getDate()).padStart(2, "0")}`;
    // Check if journal for today already exists
    const existing = appState.notes.find(
      (n) => n.type === "journal" && (n as any).date === today
    );
    if (existing) {
      await handleOpenNote(existing.id);
      return;
    }
    try {
      // Find the most recent journal for the "previous" link
      const journals = appState.notes
        .filter((n) => n.type === "journal")
        .sort((a, b) => ((b as any).date ?? "").localeCompare((a as any).date ?? ""));
      const lastJournal = journals[0];

      // Create journal note (title = today's date)
      const meta = await api.createNote(today, "journal");

      // Fill in date and previous fields
      let content = await api.readNote(meta.id);
      content = content.replace('date: ""', `date: "${today}"`);
      if (lastJournal) {
        content = content.replace('previous: ""', `previous: "@${lastJournal.id}"`);
      }
      await api.saveNote(meta.id, content);
      await refreshNotes();
      await handleOpenNote(meta.id);
    } catch (e) {
      error = `Failed to create journal: ${e}`;
    }
  }

  async function handleShowGraph() {
    try {
      const graph = await api.getGraph() as { nodes: any[]; edges: any[] };
      appState.resetEditor();
      graphData = graph;
    } catch (e) {
      error = `Failed to load graph: ${e}`;
    }
  }

  // --- Keyboard shortcuts ---

  function onKeydown(e: KeyboardEvent) {
    const mod = e.metaKey || e.ctrlKey;
    if (mod && e.key === "s") {
      e.preventDefault();
      handleSave();
    } else if (mod && e.key === "k") {
      e.preventDefault();
      appState.searchModalOpen = !appState.searchModalOpen;
    } else if (mod && e.key === "o") {
      e.preventDefault();
      handleOpenVault();
    } else if (mod && e.key === "n") {
      e.preventDefault();
      if (appState.isVaultOpen) appState.createModalOpen = true;
    }
  }

  // --- Lifecycle ---

  const currentFileUri = $derived(() => {
    if (!appState.vault || !appState.currentNoteId) return "";
    if (appState.isVaultTyp) return `file://${appState.vault.root}/vault.typ`;
    const note = appState.currentNote;
    return note ? `file://${appState.vault.root}/${note.path}` : "";
  });

  onMount(() => {
    appState.initTheme();
    document.addEventListener("keydown", onKeydown);
    window.addEventListener("beforeunload", stopLsp);
    // Auto-open last vault
    const last = appState.lastVault;
    if (last) {
      openVaultByPath(last);
    }
    return () => {
      document.removeEventListener("keydown", onKeydown);
      window.removeEventListener("beforeunload", stopLsp);
      stopLsp();
    };
  });
</script>

<div
  class="app"
  class:sidebar-hidden={!appState.sidebarOpen}
  class:preview-hidden={!appState.previewOpen || appState.isVaultTyp || graphData !== null}
>
  <Toolbar
    onOpenVault={handleOpenVault}
    onCloseVault={handleCloseVault}
    onNewNote={() => (appState.createModalOpen = true)}
    onSearch={() => (appState.searchModalOpen = true)}
    onToggleActions={() => (actionsMenuOpen = !actionsMenuOpen)}
  />

  {#if error}
    <div class="error-bar">
      {error}
      <button class="icon-btn" onclick={() => (error = "")}>×</button>
    </div>
  {/if}

  <div class="content">
    {#if appState.sidebarOpen}
      <Sidebar
        onOpenNote={handleOpenNote}
        onOpenVaultTyp={openVaultTyp}
        onShowGraph={handleShowGraph}
        onTodayJournal={handleTodayJournal}
      />
    {/if}

    <div class="editor-pane">
      {#if graphData}
        <GraphView
          nodes={graphData.nodes}
          edges={graphData.edges}
          onNavigate={(id) => { graphData = null; handleOpenNote(id); }}
        />
      {:else if appState.currentNoteId}
        {#key appState.currentNoteId}
          <Editor
            content={appState.currentContent}
            onContentChange={(text) => (appState.currentContent = text)}
            notes={appState.notes}
            vimMode={appState.vimMode}
            lspClient={lspClient}
            fileUri={currentFileUri()}
            onSave={handleSave}
            onClose={() => appState.resetEditor()}
          />
        {/key}
      {:else if appState.isVaultOpen}
        <div class="empty-state">
          <p>Select a note from the sidebar or create a new one</p>
        </div>
      {:else}
        <WelcomeScreen
          onOpenVault={handleOpenVault}
          onCreateVault={handleCreateVault}
          recentVaults={appState.recentVaults}
          onOpenRecent={openVaultByPath}
        />
      {/if}
    </div>

    {#if appState.previewOpen && appState.currentNoteId && !appState.isVaultTyp}
      <Preview onNavigate={handleOpenNote} />
    {/if}
  </div>

  {#if actionsMenuOpen}
    <ActionsMenu
      onClose={() => (actionsMenuOpen = false)}
      onSave={handleSave}
      onDelete={handleDelete}
      onRename={() => {
        actionsMenuOpen = false;
        appState.renameModalOpen = true;
      }}
      onExportPdf={handleExportPdf}
      onTogglePreview={() => (appState.previewOpen = !appState.previewOpen)}
    />
  {/if}

  {#if appState.searchModalOpen}
    <SearchModal
      onClose={() => (appState.searchModalOpen = false)}
      onOpenNote={(id) => {
        appState.searchModalOpen = false;
        handleOpenNote(id);
      }}
      onCreateNote={(title) => {
        appState.searchModalOpen = false;
        appState.prefillTitle = title;
        appState.createModalOpen = true;
      }}
    />
  {/if}

  {#if appState.createModalOpen}
    <CreateNoteModal
      onClose={() => (appState.createModalOpen = false)}
      onCreate={handleCreateNote}
    />
  {/if}

  {#if appState.renameModalOpen}
    <RenameModal
      currentId={appState.currentNoteId ?? ""}
      onClose={() => (appState.renameModalOpen = false)}
      onRename={handleRename}
    />
  {/if}
</div>

<style>
  .app {
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .content {
    flex: 1;
    display: grid;
    grid-template-columns: var(--sidebar-w) 1fr 1fr;
    overflow: hidden;
  }

  .sidebar-hidden .content {
    grid-template-columns: 0 1fr 1fr;
  }

  .preview-hidden .content {
    grid-template-columns: var(--sidebar-w) 1fr;
  }

  .sidebar-hidden.preview-hidden .content {
    grid-template-columns: 0 1fr;
  }

  .editor-pane {
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary);
    gap: 12px;
  }

  .error-bar {
    background: rgba(180, 65, 51, 0.06);
    border-bottom: 1px solid rgba(180, 65, 51, 0.15);
    color: var(--danger);
    padding: 6px 12px;
    font-size: 13px;
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
</style>
