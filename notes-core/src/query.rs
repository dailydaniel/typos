use crate::error::NotesError;
use crate::types::NoteMetadata;
use crate::vault::Vault;
use std::fs;

impl Vault {
    /// Full-text search across note titles and file content.
    pub fn search(&self, query: &str) -> Result<Vec<NoteMetadata>, NotesError> {
        let index = self.index.as_ref().ok_or_else(|| NotesError::NoteNotFound(
            "Index not loaded. Call load_index() or build_index() first.".to_string(),
        ))?;

        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        for note in &index.notes {
            // Search in title
            if note.title.to_lowercase().contains(&query_lower) {
                results.push(note.clone());
                continue;
            }
            // Search in id
            if note.id.to_lowercase().contains(&query_lower) {
                results.push(note.clone());
                continue;
            }
            // Search in tags
            if note.tags.iter().any(|t| t.to_lowercase().contains(&query_lower)) {
                results.push(note.clone());
                continue;
            }
            // Search in file content
            let abs_path = self.config.root.join(&note.path);
            if abs_path.exists() {
                if let Ok(content) = fs::read_to_string(&abs_path) {
                    if content.to_lowercase().contains(&query_lower) {
                        results.push(note.clone());
                    }
                }
            }
        }

        Ok(results)
    }

    /// List notes, optionally filtered by type.
    pub fn list_notes(&self, note_type: Option<&str>) -> Result<Vec<NoteMetadata>, NotesError> {
        let index = self.index.as_ref().ok_or_else(|| NotesError::NoteNotFound(
            "Index not loaded. Call load_index() or build_index() first.".to_string(),
        ))?;

        let notes = match note_type {
            Some(t) => index.notes.iter().filter(|n| n.note_type == t).cloned().collect(),
            None => index.notes.clone(),
        };

        Ok(notes)
    }

    /// Get notes that link TO the given note id.
    pub fn backlinks(&self, id: &str) -> Result<Vec<NoteMetadata>, NotesError> {
        let index = self.index.as_ref().ok_or_else(|| NotesError::NoteNotFound(
            "Index not loaded. Call load_index() or build_index() first.".to_string(),
        ))?;

        // Find all source ids that link to this id
        let source_ids: Vec<&str> = index
            .links
            .iter()
            .filter(|l| l.target == id)
            .map(|l| l.source.as_str())
            .collect();

        let results: Vec<NoteMetadata> = index
            .notes
            .iter()
            .filter(|n| source_ids.contains(&n.id.as_str()))
            .cloned()
            .collect();

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_vault_with_notes() -> (tempfile::TempDir, Vault) {
        let dir = tempfile::tempdir().unwrap();
        let vault_path = dir.path().join("test-vault");
        fs::create_dir_all(&vault_path).unwrap();
        let mut vault = Vault::init(&vault_path).unwrap();

        vault
            .new_note("Rust Basics", "note", None, None, &["rust", "programming"], &[])
            .unwrap();
        vault
            .new_note("Build MVP", "task", None, None, &["dev"], &[])
            .unwrap();

        // Add xlink from build-mvp to welcome
        let note_path = vault_path.join("notes/build-mvp.typ");
        let content = fs::read_to_string(&note_path).unwrap();
        fs::write(&note_path, format!("{}\n#xlink(\"welcome\")\n", content)).unwrap();

        vault.build_index().unwrap();
        (dir, vault)
    }

    #[test]
    fn test_search_by_title() {
        let (_dir, vault) = setup_vault_with_notes();
        let results = vault.search("Rust").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "rust-basics");
    }

    #[test]
    fn test_search_by_tag() {
        let (_dir, vault) = setup_vault_with_notes();
        let results = vault.search("programming").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "rust-basics");
    }

    #[test]
    fn test_list_all() {
        let (_dir, vault) = setup_vault_with_notes();
        let all = vault.list_notes(None).unwrap();
        assert_eq!(all.len(), 3); // welcome + rust-basics + build-mvp
    }

    #[test]
    fn test_list_by_type() {
        let (_dir, vault) = setup_vault_with_notes();
        let tasks = vault.list_notes(Some("task")).unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].id, "build-mvp");
    }

    #[test]
    fn test_backlinks() {
        let (_dir, vault) = setup_vault_with_notes();
        let bl = vault.backlinks("welcome").unwrap();
        assert_eq!(bl.len(), 1);
        assert_eq!(bl[0].id, "build-mvp");
    }

    #[test]
    fn test_backlinks_empty() {
        let (_dir, vault) = setup_vault_with_notes();
        let bl = vault.backlinks("rust-basics").unwrap();
        assert!(bl.is_empty());
    }
}
