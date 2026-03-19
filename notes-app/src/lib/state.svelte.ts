import type { VaultInfo, VaultTypeInfo, NoteMetadata } from "./types";

const STARRED_KEY = "typst-notes-starred";
const LAST_VAULT_KEY = "typst-notes-last-vault";
const VIM_MODE_KEY = "typst-notes-vim-mode";

function loadStarred(): Set<string> {
  try {
    const raw = localStorage.getItem(STARRED_KEY);
    if (raw) return new Set(JSON.parse(raw));
  } catch {}
  return new Set();
}

function saveStarred(ids: Set<string>) {
  localStorage.setItem(STARRED_KEY, JSON.stringify([...ids]));
}

export class AppState {
  // --- Vault ---
  vault = $state<VaultInfo | null>(null);
  notes = $state<NoteMetadata[]>([]);
  vaultTypes = $state<VaultTypeInfo[]>([]);

  // --- Editor ---
  currentNoteId = $state<string | null>(null);
  currentContent = $state("");
  originalContent = $state("");
  isVaultTyp = $state(false); // editing vault.typ (not a real note)

  // --- Derived ---
  isDirty = $derived(this.currentContent !== this.originalContent);
  currentNote = $derived(this.notes.find((n) => n.id === this.currentNoteId) ?? null);
  isVaultOpen = $derived(this.vault !== null);

  // --- UI ---
  sidebarOpen = $state(true);
  previewOpen = $state(true);
  searchModalOpen = $state(false);
  createModalOpen = $state(false);
  renameModalOpen = $state(false);
  prefillTitle = $state("");

  // --- Starred ---
  private _starredIds = $state<Set<string>>(loadStarred());

  get starredIds(): Set<string> {
    return this._starredIds;
  }

  starredNotes = $derived(this.notes.filter((n) => this._starredIds.has(n.id)));

  // --- Vim mode ---
  vimMode = $state(localStorage.getItem(VIM_MODE_KEY) === "true");

  toggleVimMode() {
    this.vimMode = !this.vimMode;
    localStorage.setItem(VIM_MODE_KEY, String(this.vimMode));
  }

  // --- Preview ---
  previewHtml = $state("");
  previewLoading = $state(false);

  // --- Methods ---

  toggleStar(id: string) {
    const next = new Set(this._starredIds);
    if (next.has(id)) {
      next.delete(id);
    } else {
      next.add(id);
    }
    this._starredIds = next;
    saveStarred(next);
  }

  isStarred(id: string): boolean {
    return this._starredIds.has(id);
  }

  get lastVault(): string | null {
    return localStorage.getItem(LAST_VAULT_KEY);
  }

  set lastVault(path: string | null) {
    if (path) {
      localStorage.setItem(LAST_VAULT_KEY, path);
    } else {
      localStorage.removeItem(LAST_VAULT_KEY);
    }
  }

  markSaved() {
    this.originalContent = this.currentContent;
  }

  resetEditor() {
    this.currentNoteId = null;
    this.currentContent = "";
    this.originalContent = "";
    this.isVaultTyp = false;
    this.previewHtml = "";
  }
}

export const appState = new AppState();
