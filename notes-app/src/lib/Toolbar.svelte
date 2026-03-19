<script lang="ts">
  import { appState } from "./state.svelte";

  interface Props {
    onOpenVault: () => void;
    onNewNote: () => void;
    onSearch: () => void;
    onToggleActions: () => void;
  }

  let { onOpenVault, onNewNote, onSearch, onToggleActions }: Props = $props();
</script>

<div class="toolbar">
  <div class="toolbar-left">
    <button
      class="icon-btn"
      title="Toggle sidebar"
      onclick={() => (appState.sidebarOpen = !appState.sidebarOpen)}
    >≡</button>
    <button onclick={onOpenVault} title="Open Vault (⌘O)">Open</button>
    {#if appState.isVaultOpen}
      <button onclick={onNewNote} title="New Note (⌘N)">New</button>
      <button onclick={onSearch} title="Search (⌘K)">Search</button>
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
    <div class="vim-switch" title={appState.vimMode ? "Disable Vim mode" : "Enable Vim mode"}>
      <span class="vim-label">VIM</span>
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
      >&#x25C9;</button>
    {/if}
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
  }
  .icon-btn:hover {
    background: var(--bg-secondary);
    border-radius: var(--radius-sm);
  }
  .icon-btn.active {
    color: var(--accent);
  }

  /* iOS-style toggle switch */
  .vim-switch {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .vim-label {
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
    background: #ccc;
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
    background: white;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.2);
    transition: transform 0.2s;
  }
  .switch-track.active .switch-thumb {
    transform: translateX(14px);
  }
</style>
