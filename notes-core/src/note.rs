use crate::csv_registry;
use crate::error::NotesError;
use crate::types::NoteMetadata;
use crate::vault::Vault;
use std::fs;

impl Vault {
    /// Create a new note file and register it in CSV.
    pub fn new_note(
        &self,
        title: &str,
        note_type: &str,
        id: Option<&str>,
        parent: Option<&str>,
        tags: &[&str],
        extra_fields: &[(&str, &str)],
    ) -> Result<NoteMetadata, NotesError> {
        let note_id = match id {
            Some(id) => id.to_string(),
            None => self.generate_id(title)?,
        };

        let content = generate_note_content(&note_id, title, note_type, parent, tags, extra_fields);
        let rel_path = format!("notes/{}.typ", note_id);
        let abs_path = self.config.root.join(&rel_path);

        // Ensure notes dir exists
        fs::create_dir_all(&self.config.notes_dir)?;
        fs::write(&abs_path, content)?;

        // Register in CSV
        csv_registry::add_note_path(&self.config.note_paths_file, &rel_path)?;

        let created = chrono::Utc::now().to_rfc3339();
        Ok(NoteMetadata {
            id: note_id,
            title: title.to_string(),
            note_type: note_type.to_string(),
            parent: parent.map(|s| s.to_string()),
            tags: tags.iter().map(|s| s.to_string()).collect(),
            created: Some(created),
            path: rel_path,
            extra: extra_fields
                .iter()
                .map(|(k, v)| (k.to_string(), serde_json::Value::String(v.to_string())))
                .collect(),
        })
    }

    /// Generate a unique ID from title (slugify + uniqueness check).
    fn generate_id(&self, title: &str) -> Result<String, NotesError> {
        let base = slug::slugify(title);
        let existing = self.note_paths().unwrap_or_default();

        if !existing.iter().any(|p| p == &format!("notes/{}.typ", base)) {
            return Ok(base);
        }

        for i in 2..1000 {
            let candidate = format!("{}-{}", base, i);
            if !existing
                .iter()
                .any(|p| p == &format!("notes/{}.typ", candidate))
            {
                return Ok(candidate);
            }
        }

        Err(NotesError::DuplicateId(base))
    }

    /// Delete a note: remove file + CSV entry.
    pub fn delete_note(&self, id: &str) -> Result<(), NotesError> {
        let rel_path = format!("notes/{}.typ", id);
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
    id: &str,
    title: &str,
    note_type: &str,
    parent: Option<&str>,
    tags: &[&str],
    extra_fields: &[(&str, &str)],
) -> String {
    let mut args = vec![
        format!("  id: \"{}\"", id),
        format!("  title: \"{}\"", title),
    ];

    if let Some(p) = parent {
        args.push(format!("  parent: \"{}\"", p));
    }

    if !tags.is_empty() {
        let tags_str: Vec<String> = tags.iter().map(|t| format!("\"{}\"", t)).collect();
        args.push(format!("  tags: ({})", tags_str.join(", ")));
    }

    for (k, v) in extra_fields {
        args.push(format!("  {}: \"{}\"", k, v));
    }

    let args_str = args.join(",\n");

    format!(
        r#"#import "../vault.typ": *

#show: {note_type}.with(
{args_str},
)

= {title}

"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_note() {
        let dir = tempfile::tempdir().unwrap();
        let vault_path = dir.path().join("test-vault");
        fs::create_dir_all(&vault_path).unwrap();
        let vault = Vault::init(&vault_path).unwrap();

        let meta = vault
            .new_note("My Task", "task", None, None, &["dev"], &[])
            .unwrap();

        assert_eq!(meta.id, "my-task");
        assert_eq!(meta.note_type, "task");
        assert_eq!(meta.tags, vec!["dev"]);
        assert!(vault_path.join("notes/my-task.typ").exists());

        // Check CSV has both welcome and new note
        let paths = vault.note_paths().unwrap();
        assert!(paths.contains(&"notes/welcome.typ".to_string()));
        assert!(paths.contains(&"notes/my-task.typ".to_string()));
    }

    #[test]
    fn test_new_note_with_custom_id() {
        let dir = tempfile::tempdir().unwrap();
        let vault_path = dir.path().join("test-vault");
        fs::create_dir_all(&vault_path).unwrap();
        let vault = Vault::init(&vault_path).unwrap();

        let meta = vault
            .new_note("Some Title", "note", Some("custom-id"), None, &[], &[])
            .unwrap();
        assert_eq!(meta.id, "custom-id");
        assert!(vault_path.join("notes/custom-id.typ").exists());
    }

    #[test]
    fn test_new_note_id_collision() {
        let dir = tempfile::tempdir().unwrap();
        let vault_path = dir.path().join("test-vault");
        fs::create_dir_all(&vault_path).unwrap();
        let vault = Vault::init(&vault_path).unwrap();

        vault.new_note("Test", "note", None, None, &[], &[]).unwrap();
        let meta2 = vault.new_note("Test", "note", None, None, &[], &[]).unwrap();
        assert_eq!(meta2.id, "test-2");
    }

    #[test]
    fn test_delete_note() {
        let dir = tempfile::tempdir().unwrap();
        let vault_path = dir.path().join("test-vault");
        fs::create_dir_all(&vault_path).unwrap();
        let vault = Vault::init(&vault_path).unwrap();

        vault.new_note("To Delete", "note", None, None, &[], &[]).unwrap();
        assert!(vault_path.join("notes/to-delete.typ").exists());

        vault.delete_note("to-delete").unwrap();
        assert!(!vault_path.join("notes/to-delete.typ").exists());

        let paths = vault.note_paths().unwrap();
        assert!(!paths.contains(&"notes/to-delete.typ".to_string()));
    }

    #[test]
    fn test_generated_content_is_parseable() {
        let content = generate_note_content(
            "test-id",
            "Test Title",
            "task",
            Some("parent-id"),
            &["tag1", "tag2"],
            &[("priority", "high")],
        );

        // Verify AST extraction works on generated content
        let result = crate::ast::extract_from_file(&content, "notes/test-id.typ").unwrap();
        let meta = result.metadata.unwrap();
        assert_eq!(meta.id, "test-id");
        assert_eq!(meta.title, "Test Title");
        assert_eq!(meta.note_type, "task");
        assert_eq!(meta.parent, Some("parent-id".to_string()));
        assert_eq!(meta.tags, vec!["tag1", "tag2"]);
    }
}
