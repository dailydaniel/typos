use crate::ast;
use crate::error::NotesError;
use crate::types::{NoteLink, NotesIndex};
use crate::vault::Vault;
use std::fs;
use std::path::Path;

impl Vault {
    /// Rebuild the entire index from all registered notes.
    /// Returns the number of indexed notes.
    pub fn build_index(&mut self) -> Result<usize, NotesError> {
        let paths = self.note_paths()?;
        let mut notes = Vec::new();
        let mut links = Vec::new();

        for rel_path in &paths {
            let abs_path = self.config.root.join(rel_path);
            if !abs_path.exists() {
                continue;
            }
            let source = fs::read_to_string(&abs_path)?;
            let extraction = ast::extract_from_file(&source, rel_path)?;

            if let Some(meta) = extraction.metadata {
                let source_id = meta.id.clone();
                let source_path = meta.path.clone();
                notes.push(meta);

                for target_id in extraction.links {
                    links.push(NoteLink {
                        source: source_id.clone(),
                        target: target_id,
                        source_path: source_path.clone(),
                    });
                }
            }
        }

        let count = notes.len();
        let index = NotesIndex {
            version: 1,
            generated_at: chrono::Utc::now().to_rfc3339(),
            notes,
            links,
        };

        self.write_index(&index)?;
        self.index = Some(index);
        Ok(count)
    }

    /// Incrementally update index for a single file.
    pub fn update_index_for_file(&mut self, path: &Path) -> Result<(), NotesError> {
        let rel_path = path
            .strip_prefix(&self.config.root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");

        // Load current index
        self.load_index()?;
        let mut index = self.index.take().unwrap();

        // Remove old entries for this file
        index.notes.retain(|n| n.path != rel_path);
        index.links.retain(|l| l.source_path != rel_path);

        // Parse and add new entries
        let abs_path = self.config.root.join(&rel_path);
        if abs_path.exists() {
            let source = fs::read_to_string(&abs_path)?;
            let extraction = ast::extract_from_file(&source, &rel_path)?;

            if let Some(meta) = extraction.metadata {
                let source_id = meta.id.clone();
                let source_path = meta.path.clone();
                index.notes.push(meta);

                for target_id in extraction.links {
                    index.links.push(NoteLink {
                        source: source_id.clone(),
                        target: target_id,
                        source_path: source_path.clone(),
                    });
                }
            }
        }

        index.generated_at = chrono::Utc::now().to_rfc3339();
        self.write_index(&index)?;
        self.index = Some(index);
        Ok(())
    }

    /// Write index to disk.
    fn write_index(&self, index: &NotesIndex) -> Result<(), NotesError> {
        let json = serde_json::to_string_pretty(index)?;
        fs::write(&self.config.index_file, json)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_index() {
        let dir = tempfile::tempdir().unwrap();
        let vault_path = dir.path().join("test-vault");
        fs::create_dir_all(&vault_path).unwrap();
        let mut vault = Vault::init(&vault_path).unwrap();

        // Add a note with xlink to welcome
        vault
            .new_note("Linked Note", "note", None, None, &[], &[])
            .unwrap();

        // Manually add xlink content
        let note_path = vault_path.join("notes/linked-note.typ");
        let content = fs::read_to_string(&note_path).unwrap();
        let content = format!("{}\n#xlink(\"welcome\")\n", content);
        fs::write(&note_path, content).unwrap();

        let count = vault.build_index().unwrap();
        assert_eq!(count, 2); // welcome + linked-note

        let index = vault.index.as_ref().unwrap();
        assert_eq!(index.notes.len(), 2);
        assert_eq!(index.links.len(), 1);
        assert_eq!(index.links[0].source, "linked-note");
        assert_eq!(index.links[0].target, "welcome");
    }

    #[test]
    fn test_incremental_update() {
        let dir = tempfile::tempdir().unwrap();
        let vault_path = dir.path().join("test-vault");
        fs::create_dir_all(&vault_path).unwrap();
        let mut vault = Vault::init(&vault_path).unwrap();

        vault.build_index().unwrap();
        assert_eq!(vault.index.as_ref().unwrap().notes.len(), 1);

        // Add a note
        vault
            .new_note("New Note", "note", None, None, &[], &[])
            .unwrap();

        // Incrementally update just the new note
        let note_path = vault_path.join("notes/new-note.typ");
        vault.update_index_for_file(&note_path).unwrap();

        let index = vault.index.as_ref().unwrap();
        assert_eq!(index.notes.len(), 2);
    }
}
