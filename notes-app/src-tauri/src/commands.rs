use crate::state::AppState;
use notes_core::types::{self, NoteMetadata, VaultType};
use notes_core::vault::Vault;
use serde::Serialize;
use std::fs;
use std::path::Path;
use tauri::State;

#[derive(Serialize)]
pub struct VaultInfo {
    pub root: String,
    pub note_count: usize,
    pub types: Vec<VaultTypeInfo>,
}

#[derive(Serialize)]
pub struct VaultTypeInfo {
    pub name: String,
    pub fields: Vec<(String, String)>,
}

impl From<VaultType> for VaultTypeInfo {
    fn from(vt: VaultType) -> Self {
        Self {
            name: vt.name,
            fields: vt.fields,
        }
    }
}

fn with_vault<F, T>(state: &State<AppState>, f: F) -> Result<T, String>
where
    F: FnOnce(&Vault) -> Result<T, String>,
{
    let guard = state.vault.lock().map_err(|e| e.to_string())?;
    let vault = guard.as_ref().ok_or("No vault opened")?;
    f(vault)
}

fn with_vault_mut<F, T>(state: &State<AppState>, f: F) -> Result<T, String>
where
    F: FnOnce(&mut Vault) -> Result<T, String>,
{
    let mut guard = state.vault.lock().map_err(|e| e.to_string())?;
    let vault = guard.as_mut().ok_or("No vault opened")?;
    f(vault)
}

/// Apply bundled typst binary and package paths to a vault.
fn apply_bundled_paths(state: &State<AppState>, vault: &mut Vault) {
    if let Ok(bin) = state.typst_binary.lock() {
        vault.typst_binary = bin.clone();
    }
    if let Ok(pkg) = state.package_path.lock() {
        vault.package_path = pkg.clone();
    }
}

// --- Vault management ---

#[tauri::command]
pub fn open_vault(state: State<AppState>, path: String) -> Result<VaultInfo, String> {
    let p = Path::new(&path);
    let mut vault = Vault::open(p).map_err(|e| e.to_string())?;
    vault.build_index().map_err(|e| e.to_string())?;

    let note_count = vault.index.as_ref().map(|i| i.notes.len()).unwrap_or(0);
    let vault_types: Vec<VaultTypeInfo> = vault
        .note_types()
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|t| t.into())
        .collect();

    let root = vault.config.root.display().to_string();

    apply_bundled_paths(&state, &mut vault);
    *state.vault.lock().map_err(|e| e.to_string())? = Some(vault);

    Ok(VaultInfo {
        root,
        note_count,
        types: vault_types,
    })
}

#[tauri::command]
pub fn init_vault(state: State<AppState>, path: String) -> Result<VaultInfo, String> {
    let p = Path::new(&path);
    let mut vault = Vault::init(p).map_err(|e| e.to_string())?;

    let note_count = vault.index.as_ref().map(|i| i.notes.len()).unwrap_or(0);
    let vault_types: Vec<VaultTypeInfo> = vault
        .note_types()
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|t| t.into())
        .collect();

    let root = vault.config.root.display().to_string();

    apply_bundled_paths(&state, &mut vault);
    *state.vault.lock().map_err(|e| e.to_string())? = Some(vault);

    Ok(VaultInfo {
        root,
        note_count,
        types: vault_types,
    })
}

#[tauri::command]
pub fn get_vault_types(state: State<AppState>) -> Result<Vec<VaultTypeInfo>, String> {
    with_vault(&state, |vault| {
        vault
            .note_types()
            .map(|types| types.into_iter().map(|t| t.into()).collect())
            .map_err(|e| e.to_string())
    })
}

// --- Notes CRUD ---

#[tauri::command]
pub fn list_notes(
    state: State<AppState>,
    note_type: Option<String>,
) -> Result<Vec<NoteMetadata>, String> {
    with_vault(&state, |vault| {
        vault
            .list_notes(note_type.as_deref())
            .map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn create_note(
    state: State<AppState>,
    title: String,
    note_type: String,
) -> Result<NoteMetadata, String> {
    with_vault(&state, |vault| {
        vault
            .new_note(&title, &note_type, &[])
            .map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn delete_note(state: State<AppState>, id: String) -> Result<(), String> {
    with_vault(&state, |vault| {
        vault.delete_note(&id).map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn rename_note(
    state: State<AppState>,
    old_id: String,
    new_id: String,
) -> Result<Vec<String>, String> {
    with_vault(&state, |vault| {
        vault
            .rename_note(&old_id, &new_id)
            .map_err(|e| e.to_string())
    })
}

// --- File read/write ---

#[tauri::command]
pub fn read_note(state: State<AppState>, id: String) -> Result<String, String> {
    with_vault(&state, |vault| {
        let rel_path = types::id_to_path(&id);
        let abs_path = vault.config.root.join(&rel_path);
        fs::read_to_string(&abs_path).map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn save_note(state: State<AppState>, id: String, content: String) -> Result<(), String> {
    with_vault_mut(&state, |vault| {
        let rel_path = types::id_to_path(&id);
        let abs_path = vault.config.root.join(&rel_path);
        fs::write(&abs_path, &content).map_err(|e| e.to_string())?;
        let path = vault.config.root.join(&rel_path);
        vault
            .update_index_for_file(&path)
            .map_err(|e| e.to_string())
    })
}

// --- Compile ---

#[tauri::command]
pub fn compile_note(state: State<AppState>, id: String) -> Result<String, String> {
    with_vault_mut(&state, |vault| {
        let rel_path = types::id_to_path(&id);
        let note_path = vault.config.root.join(&rel_path);
        let output_path = vault.default_output_path("html");

        vault
            .compile_note(&note_path, &output_path, "html")
            .map_err(|e| e.to_string())?;

        fs::read_to_string(&output_path).map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn compile_note_pdf(
    state: State<AppState>,
    id: String,
    output: String,
) -> Result<String, String> {
    with_vault_mut(&state, |vault| {
        let rel_path = types::id_to_path(&id);
        let note_path = vault.config.root.join(&rel_path);
        let output_path = Path::new(&output);

        vault
            .reindex_if_stale()
            .map_err(|e| e.to_string())?;
        vault
            .compile_note(&note_path, output_path, "pdf")
            .map_err(|e| e.to_string())?;

        Ok(output)
    })
}

// --- Search & navigation ---

#[tauri::command]
pub fn search_notes(state: State<AppState>, query: String) -> Result<Vec<NoteMetadata>, String> {
    with_vault(&state, |vault| {
        vault.search(&query).map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn get_backlinks(state: State<AppState>, id: String) -> Result<Vec<NoteMetadata>, String> {
    with_vault(&state, |vault| {
        vault.backlinks(&id).map_err(|e| e.to_string())
    })
}

// --- Graph ---

#[tauri::command]
pub fn get_graph(state: State<AppState>) -> Result<serde_json::Value, String> {
    with_vault(&state, |vault| {
        let graph = vault.graph_data().map_err(|e| e.to_string())?;
        serde_json::to_value(&graph).map_err(|e| e.to_string())
    })
}

// --- Index management ---

#[tauri::command]
pub fn reindex(state: State<AppState>) -> Result<usize, String> {
    with_vault_mut(&state, |vault| {
        vault.build_index().map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn sync_vault(state: State<AppState>) -> Result<(usize, usize), String> {
    with_vault_mut(&state, |vault| {
        vault.sync().map_err(|e| e.to_string())
    })
}
