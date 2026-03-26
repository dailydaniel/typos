import type { VaultInfo, VaultTypeInfo, NoteMetadata } from "./types";

const STARRED_KEY = "typos-starred";
const LAST_VAULT_KEY = "typos-last-vault";
const RECENT_VAULTS_KEY = "typos-recent-vaults";
const DARK_MODE_KEY = "typos-dark-mode";
const VIM_MODE_KEY = "typos-vim-mode";
const PREVIEW_FORMAT_KEY = "typos-preview-format";

// Migrate from old keys
function migrateLocalStorage() {
  const migrations: [string, string][] = [
    ["typst-notes-starred", STARRED_KEY],
    ["typst-notes-last-vault", LAST_VAULT_KEY],
    ["typst-notes-vim-mode", VIM_MODE_KEY],
  ];
  for (const [oldKey, newKey] of migrations) {
    const val = localStorage.getItem(oldKey);
    if (val !== null && localStorage.getItem(newKey) === null) {
      localStorage.setItem(newKey, val);
      localStorage.removeItem(oldKey);
    }
  }
}
migrateLocalStorage();

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

  // --- Dark mode ---
  darkMode = $state(localStorage.getItem(DARK_MODE_KEY) === "true");

  toggleDarkMode() {
    this.darkMode = !this.darkMode;
    localStorage.setItem(DARK_MODE_KEY, String(this.darkMode));
    document.documentElement.setAttribute("data-theme", this.darkMode ? "dark" : "light");
  }

  initTheme() {
    document.documentElement.setAttribute("data-theme", this.darkMode ? "dark" : "light");
  }

  // --- Vim mode ---
  vimMode = $state(localStorage.getItem(VIM_MODE_KEY) === "true");

  toggleVimMode() {
    this.vimMode = !this.vimMode;
    localStorage.setItem(VIM_MODE_KEY, String(this.vimMode));
  }

  // --- Preview ---
  previewHtml = $state("");
  previewLoading = $state(false);
  previewFormat = $state<"html" | "pdf">(
    (localStorage.getItem(PREVIEW_FORMAT_KEY) as "html" | "pdf") || "html"
  );

  togglePreviewFormat() {
    this.previewFormat = this.previewFormat === "html" ? "pdf" : "html";
    localStorage.setItem(PREVIEW_FORMAT_KEY, this.previewFormat);
  }

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
      this.addRecentVault(path);
    } else {
      localStorage.removeItem(LAST_VAULT_KEY);
    }
  }

  get recentVaults(): string[] {
    try {
      const raw = localStorage.getItem(RECENT_VAULTS_KEY);
      if (raw) return JSON.parse(raw);
    } catch {}
    // Seed from lastVault if recent list doesn't exist yet
    const last = localStorage.getItem(LAST_VAULT_KEY);
    return last ? [last] : [];
  }

  addRecentVault(path: string) {
    const recent = this.recentVaults.filter((p) => p !== path);
    recent.unshift(path);
    const trimmed = recent.slice(0, 5);
    localStorage.setItem(RECENT_VAULTS_KEY, JSON.stringify(trimmed));
  }

  removeRecentVault(path: string) {
    const recent = this.recentVaults.filter((p) => p !== path);
    localStorage.setItem(RECENT_VAULTS_KEY, JSON.stringify(recent));
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
