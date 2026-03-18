use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Metadata extracted from a note's AST
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteMetadata {
    pub id: String,
    pub title: String,
    #[serde(rename = "type")]
    pub note_type: String,
    pub parent: Option<String>,
    pub created: Option<String>,
    pub path: String,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// A link between notes (from xlink call)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteLink {
    pub source: String,
    pub target: String,
    pub source_path: String,
}

/// Complete vault index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotesIndex {
    pub version: u32,
    pub generated_at: String,
    pub notes: Vec<NoteMetadata>,
    pub links: Vec<NoteLink>,
}

/// Vault configuration (paths)
#[derive(Debug, Clone)]
pub struct VaultConfig {
    pub root: PathBuf,
    pub note_paths_file: PathBuf,
    pub index_file: PathBuf,
    pub notes_dir: PathBuf,
    pub assets_dir: PathBuf,
}

/// Convert a relative file path to a note id.
/// "notes/programming--rust--closures.typ" → "programming/rust/closures"
pub fn path_to_id(rel_path: &str) -> String {
    rel_path
        .trim_start_matches("notes/")
        .trim_end_matches(".typ")
        .replace("--", "/")
}

/// Convert a note id to a relative file path.
/// "programming/rust/closures" → "notes/programming--rust--closures.typ"
pub fn id_to_path(id: &str) -> String {
    format!("notes/{}.typ", id.replace("/", "--"))
}

/// Get the parent id from a note id.
/// "programming/rust/closures" → Some("programming/rust")
/// "programming" → None
pub fn id_to_parent(id: &str) -> Option<String> {
    id.rfind('/').map(|i| id[..i].to_string())
}
