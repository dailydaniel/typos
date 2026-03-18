use crate::csv_registry;
use crate::error::NotesError;
use crate::types::{self, NoteMetadata};
use crate::vault::Vault;
use std::fs;

impl Vault {
    /// Create a new note with type validation against vault.typ.
    /// Supports hierarchical paths: "a/b/c" creates parent notes automatically.
    /// Returns metadata of the target note (last segment).
    pub fn new_note(
        &self,
        path_title: &str,
        note_type: &str,
        extra_fields: &[(&str, &str)],
    ) -> Result<NoteMetadata, NotesError> {
        // Parse vault.typ for type definitions
        let vault_types = self.note_types()?;

        // Validate type
        let target_type = vault_types.iter().find(|t| t.name == note_type);
        if target_type.is_none() {
            let available: Vec<&str> = vault_types.iter().map(|t| t.name.as_str()).collect();
            return Err(NotesError::InvalidNoteType(format!(
                "unknown note type \"{}\". Available types: {}",
                note_type,
                available.join(", ")
            )));
        }

        let segments: Vec<&str> = path_title.split('/').collect();

        // Create parent notes if they don't exist
        let parent_type = vault_types.iter().find(|t| t.name == "note");
        let parent_fields: Vec<(&str, &str)> = parent_type
            .map(|t| t.fields.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect())
            .unwrap_or_default();

        for i in 0..segments.len().saturating_sub(1) {
            let parent_segments = &segments[..=i];
            let parent_id = parent_segments
                .iter()
                .map(|s| slug::slugify(s))
                .collect::<Vec<_>>()
                .join("/");
            let parent_path = types::id_to_path(&parent_id);

            if !self.config.root.join(&parent_path).exists() {
                let title = segments[i].to_string();
                let content = generate_note_content(&title, "note", &parent_fields);
                fs::create_dir_all(&self.config.notes_dir)?;
                fs::write(self.config.root.join(&parent_path), content)?;
                csv_registry::add_note_path(&self.config.note_paths_file, &parent_path)?;
            }
        }

        // Build the full id
        let full_id = segments
            .iter()
            .map(|s| slug::slugify(s))
            .collect::<Vec<_>>()
            .join("/");
        let rel_path = types::id_to_path(&full_id);
        let abs_path = self.config.root.join(&rel_path);
        let title = segments.last().unwrap().to_string();

        if abs_path.exists() {
            return Err(NotesError::DuplicateId(full_id));
        }

        fs::create_dir_all(&self.config.notes_dir)?;

        // Merge type fields with extra fields (extra overrides type defaults)
        // Type fields are raw Typst source (e.g. "()", "\"\"")
        // CLI extra fields are plain strings — wrap in quotes
        let type_fields = &target_type.unwrap().fields;
        let mut all_fields: Vec<(String, String)> = type_fields
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        for (k, v) in extra_fields {
            let quoted = format!("\"{}\"", v);
            if let Some(existing) = all_fields.iter_mut().find(|(ek, _)| ek == k) {
                existing.1 = quoted;
            } else {
                all_fields.push((k.to_string(), quoted));
            }
        }

        let fields_ref: Vec<(&str, &str)> = all_fields.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
        let content = generate_note_content(&title, note_type, &fields_ref);
        fs::write(&abs_path, content)?;
        csv_registry::add_note_path(&self.config.note_paths_file, &rel_path)?;

        let parent = types::id_to_parent(&full_id);

        Ok(NoteMetadata {
            id: full_id,
            title,
            note_type: note_type.to_string(),
            parent,
            created: Some(chrono::Utc::now().to_rfc3339()),
            path: rel_path,
            extra: all_fields
                .iter()
                .map(|(k, v)| (k.to_string(), serde_json::Value::String(v.to_string())))
                .collect(),
        })
    }

    /// Rename a note and update all references.
    /// Also renames children (notes with id prefix).
    pub fn rename_note(&self, old_id: &str, new_id: &str) -> Result<Vec<String>, NotesError> {
        let old_path = types::id_to_path(old_id);
        let old_abs = self.config.root.join(&old_path);
        if !old_abs.exists() {
            return Err(NotesError::NoteNotFound(old_id.to_string()));
        }

        let new_path = types::id_to_path(new_id);
        let new_abs = self.config.root.join(&new_path);
        if new_abs.exists() {
            return Err(NotesError::DuplicateId(new_id.to_string()));
        }

        // Collect all notes to rename: the note itself + children
        let all_paths = self.note_paths()?;
        let old_prefix = format!("{}/", old_id);
        let mut renames: Vec<(String, String)> = Vec::new(); // (old_id, new_id)

        // The note itself
        renames.push((old_id.to_string(), new_id.to_string()));

        // Children: old_id/child → new_id/child
        for path in &all_paths {
            let path_id = types::path_to_id(path);
            if path_id.starts_with(&old_prefix) {
                let suffix = &path_id[old_id.len()..];
                let child_new_id = format!("{}{}", new_id, suffix);
                renames.push((path_id, child_new_id));
            }
        }

        // 1. Rename files and update CSV
        for (oid, nid) in &renames {
            let op = types::id_to_path(oid);
            let np = types::id_to_path(nid);
            let src = self.config.root.join(&op);
            let dst = self.config.root.join(&np);
            if src.exists() {
                fs::rename(&src, &dst)?;
                csv_registry::remove_note_path(&self.config.note_paths_file, &op)?;
                csv_registry::add_note_path(&self.config.note_paths_file, &np)?;
            }
        }

        // 2. Update references in ALL note files
        let updated_paths = self.note_paths()?;
        for path in &updated_paths {
            let abs = self.config.root.join(path);
            if !abs.exists() { continue; }
            let content = fs::read_to_string(&abs)?;
            let mut new_content = content.clone();

            for (oid, nid) in &renames {
                // Replace @old-id references
                new_content = new_content.replace(
                    &format!("@{}", oid),
                    &format!("@{}", nid),
                );
                // Replace xlink("old-id") references
                new_content = new_content.replace(
                    &format!("xlink(\"{}\")", oid),
                    &format!("xlink(\"{}\")", nid),
                );
            }

            if new_content != content {
                fs::write(&abs, new_content)?;
            }
        }

        let renamed_ids: Vec<String> = renames.into_iter().map(|(_, n)| n).collect();
        Ok(renamed_ids)
    }

    /// Delete a note: remove file + CSV entry.
    pub fn delete_note(&self, id: &str) -> Result<(), NotesError> {
        let rel_path = types::id_to_path(id);
        let abs_path = self.config.root.join(&rel_path);

        if !abs_path.exists() {
            return Err(NotesError::NoteNotFound(id.to_string()));
        }

        fs::remove_file(&abs_path)?;
        csv_registry::remove_note_path(&self.config.note_paths_file, &rel_path)?;
        Ok(())
    }
}

/// Generate Typst content for a new note.
fn generate_note_content(
    title: &str,
    note_type: &str,
    extra_fields: &[(&str, &str)],
) -> String {
    let mut args = vec![format!("  title: \"{}\"", title)];

    for (k, v) in extra_fields {
        // Values are raw Typst source (already include quotes for strings)
        args.push(format!("  {}: {}", k, v));
    }

    let args_str = args.join(",\n");

    format!(
        r#"#import "../vault.typ": *

#show: {note_type}.with(
{args_str},
)
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_simple_note() {
        let dir = tempfile::tempdir().unwrap();
        let vault_path = dir.path().join("test-vault");
        fs::create_dir_all(&vault_path).unwrap();
        let vault = Vault::init(&vault_path).unwrap();

        let meta = vault.new_note("My Task", "task", &[]).unwrap();

        assert_eq!(meta.id, "my-task");
        assert_eq!(meta.note_type, "task");
        assert!(meta.parent.is_none());
        assert!(vault_path.join("notes/my-task.typ").exists());
    }

    #[test]
    fn test_new_hierarchical_note() {
        let dir = tempfile::tempdir().unwrap();
        let vault_path = dir.path().join("test-vault");
        fs::create_dir_all(&vault_path).unwrap();
        let vault = Vault::init(&vault_path).unwrap();

        let meta = vault
            .new_note("programming/rust/closures", "task", &[])
            .unwrap();

        assert_eq!(meta.id, "programming/rust/closures");
        assert_eq!(meta.parent, Some("programming/rust".to_string()));
        assert_eq!(meta.title, "closures");

        // Check parent files were auto-created
        assert!(vault_path.join("notes/programming.typ").exists());
        assert!(vault_path.join("notes/programming--rust.typ").exists());
        assert!(vault_path
            .join("notes/programming--rust--closures.typ")
            .exists());

        // Check CSV has all three + welcome
        let paths = vault.note_paths().unwrap();
        assert_eq!(paths.len(), 4);
    }

    #[test]
    fn test_existing_parent_not_recreated() {
        let dir = tempfile::tempdir().unwrap();
        let vault_path = dir.path().join("test-vault");
        fs::create_dir_all(&vault_path).unwrap();
        let vault = Vault::init(&vault_path).unwrap();

        vault.new_note("programming/rust", "note", &[]).unwrap();
        vault
            .new_note("programming/rust/closures", "task", &[])
            .unwrap();

        // programming and programming/rust should exist once each in CSV
        let paths = vault.note_paths().unwrap();
        let prog_count = paths
            .iter()
            .filter(|p| p.as_str() == "notes/programming.typ")
            .count();
        assert_eq!(prog_count, 1);
    }

    #[test]
    fn test_delete_note() {
        let dir = tempfile::tempdir().unwrap();
        let vault_path = dir.path().join("test-vault");
        fs::create_dir_all(&vault_path).unwrap();
        let vault = Vault::init(&vault_path).unwrap();

        vault.new_note("to-delete", "note", &[]).unwrap();
        assert!(vault_path.join("notes/to-delete.typ").exists());

        vault.delete_note("to-delete").unwrap();
        assert!(!vault_path.join("notes/to-delete.typ").exists());
    }

    #[test]
    fn test_generated_content_is_parseable() {
        let content = generate_note_content("Test Title", "task", &[("priority", "high")]);
        let result = crate::ast::extract_from_file(&content, "notes/test-title.typ").unwrap();
        let meta = result.metadata.unwrap();
        assert_eq!(meta.id, "test-title");
        assert_eq!(meta.title, "Test Title");
        assert_eq!(meta.note_type, "task");
    }

    #[test]
    fn test_invalid_type_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let vault_path = dir.path().join("test-vault");
        fs::create_dir_all(&vault_path).unwrap();
        let vault = Vault::init(&vault_path).unwrap();

        let result = vault.new_note("banana-note", "banana", &[]);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("banana"));
        assert!(err.contains("Available types"));
    }

    #[test]
    fn test_type_fields_generated() {
        let dir = tempfile::tempdir().unwrap();
        let vault_path = dir.path().join("test-vault");
        fs::create_dir_all(&vault_path).unwrap();
        let vault = Vault::init(&vault_path).unwrap();

        vault.new_note("My Task", "task", &[]).unwrap();

        let content = fs::read_to_string(vault_path.join("notes/my-task.typ")).unwrap();
        // task type has fields: (tags: (), priority: "")
        assert!(content.contains("tags:"));
        assert!(content.contains("priority:"));
    }

    #[test]
    fn test_extra_fields_override_type_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let vault_path = dir.path().join("test-vault");
        fs::create_dir_all(&vault_path).unwrap();
        let vault = Vault::init(&vault_path).unwrap();

        vault.new_note("My Task", "task", &[("priority", "high")]).unwrap();

        let content = fs::read_to_string(vault_path.join("notes/my-task.typ")).unwrap();
        assert!(content.contains("priority: \"high\""));
    }

    #[test]
    fn test_rename_simple() {
        let dir = tempfile::tempdir().unwrap();
        let vault_path = dir.path().join("test-vault");
        fs::create_dir_all(&vault_path).unwrap();
        let vault = Vault::init(&vault_path).unwrap();

        vault.new_note("old-name", "note", &[]).unwrap();
        let renamed = vault.rename_note("old-name", "new-name").unwrap();

        assert_eq!(renamed, vec!["new-name"]);
        assert!(!vault_path.join("notes/old-name.typ").exists());
        assert!(vault_path.join("notes/new-name.typ").exists());
    }

    #[test]
    fn test_rename_with_children() {
        let dir = tempfile::tempdir().unwrap();
        let vault_path = dir.path().join("test-vault");
        fs::create_dir_all(&vault_path).unwrap();
        let vault = Vault::init(&vault_path).unwrap();

        vault.new_note("programming/rust", "note", &[]).unwrap();
        vault.new_note("programming/rust/closures", "task", &[]).unwrap();

        let renamed = vault.rename_note("programming", "dev").unwrap();

        assert_eq!(renamed.len(), 3); // dev, dev/rust, dev/rust/closures
        assert!(vault_path.join("notes/dev.typ").exists());
        assert!(vault_path.join("notes/dev--rust.typ").exists());
        assert!(vault_path.join("notes/dev--rust--closures.typ").exists());
        assert!(!vault_path.join("notes/programming.typ").exists());
    }

    #[test]
    fn test_rename_updates_references() {
        let dir = tempfile::tempdir().unwrap();
        let vault_path = dir.path().join("test-vault");
        fs::create_dir_all(&vault_path).unwrap();
        let vault = Vault::init(&vault_path).unwrap();

        vault.new_note("target", "note", &[]).unwrap();
        vault.new_note("linker", "note", &[]).unwrap();

        // Add xlink and @ref to linker
        let linker_path = vault_path.join("notes/linker.typ");
        let content = fs::read_to_string(&linker_path).unwrap();
        fs::write(&linker_path, format!(
            "{}\n#xlink(\"target\")\ntags: (\"@target\")\n",
            content
        )).unwrap();

        vault.rename_note("target", "new-target").unwrap();

        let updated = fs::read_to_string(vault_path.join("notes/linker.typ")).unwrap();
        assert!(updated.contains("xlink(\"new-target\")"));
        assert!(updated.contains("@new-target"));
        assert!(!updated.contains("xlink(\"target\")"));
        assert!(!updated.contains("@target"));
    }

    #[test]
    fn test_rename_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let vault_path = dir.path().join("test-vault");
        fs::create_dir_all(&vault_path).unwrap();
        let vault = Vault::init(&vault_path).unwrap();

        let result = vault.rename_note("nonexistent", "new-name");
        assert!(result.is_err());
    }

    #[test]
    fn test_rename_duplicate_target() {
        let dir = tempfile::tempdir().unwrap();
        let vault_path = dir.path().join("test-vault");
        fs::create_dir_all(&vault_path).unwrap();
        let vault = Vault::init(&vault_path).unwrap();

        vault.new_note("note-a", "note", &[]).unwrap();
        vault.new_note("note-b", "note", &[]).unwrap();

        let result = vault.rename_note("note-a", "note-b");
        assert!(result.is_err());
    }

    #[test]
    fn test_parent_gets_note_type_fields() {
        let dir = tempfile::tempdir().unwrap();
        let vault_path = dir.path().join("test-vault");
        fs::create_dir_all(&vault_path).unwrap();
        let vault = Vault::init(&vault_path).unwrap();

        vault.new_note("programming/rust", "task", &[]).unwrap();

        // Auto-created parent "programming" should have note type fields
        let parent_content = fs::read_to_string(vault_path.join("notes/programming.typ")).unwrap();
        assert!(parent_content.contains("tags:"));
        assert!(parent_content.contains("links:"));
    }
}
