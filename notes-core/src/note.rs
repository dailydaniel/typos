use crate::csv_registry;
use crate::error::NotesError;
use crate::types::{self, NoteMetadata};
use crate::vault::Vault;
use std::fs;

impl Vault {
    /// Create a new note. Supports hierarchical paths: "a/b/c" creates
    /// parent notes automatically if they don't exist.
    /// Returns metadata of the target note (last segment).
    pub fn new_note(
        &self,
        path_title: &str,
        note_type: &str,
        extra_fields: &[(&str, &str)],
    ) -> Result<NoteMetadata, NotesError> {
        let segments: Vec<&str> = path_title.split('/').collect();

        // Create parent notes if they don't exist
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
                let content = generate_note_content(&title, "note", &[]);
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

        fs::create_dir_all(&self.config.notes_dir)?;

        let content = generate_note_content(&title, note_type, extra_fields);
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
            extra: extra_fields
                .iter()
                .map(|(k, v)| (k.to_string(), serde_json::Value::String(v.to_string())))
                .collect(),
        })
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
        args.push(format!("  {}: \"{}\"", k, v));
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
            .new_note("programming/rust/closures", "card", &[])
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
            .new_note("programming/rust/closures", "card", &[])
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
}
