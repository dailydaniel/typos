<script lang="ts">
  import { appState } from "./state.svelte";

  interface Props {
    onOpenVault: () => void;
    onCloseVault: () => void;
    onNewNote: () => void;
    onSearch: () => void;
    onToggleActions: () => void;
  }

  let { onOpenVault, onCloseVault, onNewNote, onSearch, onToggleActions }: Props = $props();
</script>

<div class="toolbar">
  <div class="toolbar-left">
    <button
      class="icon-btn"
      title="Toggle sidebar"
      onclick={() => (appState.sidebarOpen = !appState.sidebarOpen)}
    >≡</button>
    <button class="icon-btn" onclick={onOpenVault} title="Open Vault (⌘O)">
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
        <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
      </svg>
    </button>
    {#if appState.isVaultOpen}
      <button class="icon-btn" onclick={onCloseVault} title="Close vault">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
          <line x1="18" y1="6" x2="6" y2="18"/>
          <line x1="6" y1="6" x2="18" y2="18"/>
        </svg>
      </button>
      <button class="icon-btn" onclick={onNewNote} title="New Note (⌘N)">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
          <line x1="12" y1="5" x2="12" y2="19"/>
          <line x1="5" y1="12" x2="19" y2="12"/>
        </svg>
      </button>
      <button class="icon-btn" onclick={onSearch} title="Search (⌘K)">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="11" cy="11" r="7"/>
          <line x1="16" y1="16" x2="21" y2="21"/>
        </svg>
      </button>
    {/if}
  </div>

  <div class="toolbar-center">
    {#if appState.currentNoteId}
      <span class="note-indicator">
        {appState.isVaultTyp ? "vault.typ" : appState.currentNoteId}
        {#if appState.isDirty}
          <span class="dirty-dot" title="Unsaved changes"></span>
        {/if}
      </span>
    {/if}
  </div>

  <div class="toolbar-right">
    {#if appState.currentNoteId && !appState.isVaultTyp && appState.previewOpen}
      <div class="toggle-switch" title={appState.previewFormat === "pdf" ? "Switch to HTML preview" : "Switch to PDF preview"}>
        <span class="toggle-label">PDF</span>
        <button
          class="switch-track"
          class:active={appState.previewFormat === "pdf"}
          onclick={() => appState.togglePreviewFormat()}
          role="switch"
          aria-checked={appState.previewFormat === "pdf"}
          aria-label="Toggle PDF preview"
        >
          <span class="switch-thumb"></span>
        </button>
      </div>
    {/if}
    <div class="toggle-switch" title={appState.vimMode ? "Disable Vim mode" : "Enable Vim mode"}>
      <span class="toggle-label">VIM</span>
      <button
        class="switch-track"
        class:active={appState.vimMode}
        onclick={() => appState.toggleVimMode()}
        role="switch"
        aria-checked={appState.vimMode}
        aria-label="Toggle Vim mode"
      >
        <span class="switch-thumb"></span>
      </button>
    </div>
    {#if appState.currentNoteId && !appState.isVaultTyp}
      <button
        class="icon-btn"
        class:active={appState.previewOpen}
        onclick={() => (appState.previewOpen = !appState.previewOpen)}
        title={appState.previewOpen ? "Hide preview" : "Show preview"}
      >
        {#if appState.previewOpen}
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <path d="M2 12c2.5-5 6.5-8 10-8s7.5 3 10 8c-2.5 5-6.5 8-10 8s-7.5-3-10-8z"/>
            <circle cx="12" cy="12" r="3"/>
          </svg>
        {:else}
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <path d="M2 12c2.5-5 6.5-8 10-8s7.5 3 10 8c-2.5 5-6.5 8-10 8s-7.5-3-10-8z"/>
            <circle cx="12" cy="12" r="3"/>
            <line x1="4" y1="20" x2="20" y2="4"/>
          </svg>
        {/if}
      </button>
    {/if}
    <button
      class="icon-btn"
      onclick={() => appState.toggleDarkMode()}
      title={appState.darkMode ? "Switch to light mode" : "Switch to dark mode"}
    >
      {#if appState.darkMode}
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="5"/>
          <line x1="12" y1="1" x2="12" y2="3"/>
          <line x1="12" y1="21" x2="12" y2="23"/>
          <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/>
          <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/>
          <line x1="1" y1="12" x2="3" y2="12"/>
          <line x1="21" y1="12" x2="23" y2="12"/>
          <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/>
          <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/>
        </svg>
      {:else}
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M21 12.79A9 9 0 1111.21 3 7 7 0 0021 12.79z"/>
        </svg>
      {/if}
    </button>
    {#if appState.currentNoteId}
      <button class="icon-btn" onclick={onToggleActions} title="Actions">···</button>
    {/if}
  </div>
</div>

<style>
  .toolbar {
    height: var(--toolbar-h);
    padding: 0 12px;
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
    user-select: none;
  }
  .toolbar-left {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .toolbar-center {
    flex: 1;
    text-align: center;
  }
  .toolbar-right {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .note-indicator {
    font-size: 13px;
    color: var(--text-secondary);
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
  .dirty-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--danger);
    display: inline-block;
  }
  .icon-btn {
    font-size: 18px;
    padding: 4px 8px;
    line-height: 1;
    border: none;
    background: none;
    display: flex;
    align-items: center;
  }
  .icon-btn:hover {
    background: var(--bg-secondary);
    border-radius: var(--radius-sm);
  }
  .icon-btn.active {
    color: var(--accent);
  }

  /* iOS-style toggle switch */
  .toggle-switch {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .toggle-label {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-secondary);
    letter-spacing: 0.5px;
  }
  .switch-track {
    position: relative;
    width: 32px;
    height: 18px;
    border-radius: 9px;
    border: none;
    background: var(--border);
    padding: 0;
    cursor: pointer;
    transition: background 0.2s;
  }
  .switch-track.active {
    background: var(--accent);
  }
  .switch-thumb {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: var(--surface);
    box-shadow: 0 1px 2px rgba(44, 40, 37, 0.15);
    transition: transform 0.2s;
  }
  .switch-track.active .switch-thumb {
    transform: translateX(14px);
  }
</style>
