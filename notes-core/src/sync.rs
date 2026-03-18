use crate::csv_registry;
use crate::error::NotesError;
use crate::vault::Vault;
use std::collections::HashSet;
use std::fs;

impl Vault {
    /// Sync CSV with filesystem: scan notes/*.typ, add new, remove missing.
    /// Then rebuild index. Returns (added, removed) counts.
    pub fn sync(&mut self) -> Result<(usize, usize), NotesError> {
        let csv_paths: HashSet<String> = self.note_paths()?.into_iter().collect();

        // Scan filesystem for .typ files in notes/
        let mut fs_paths: HashSet<String> = HashSet::new();
        if self.config.notes_dir.exists() {
            for entry in fs::read_dir(&self.config.notes_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("typ") {
                    let rel = format!(
                        "notes/{}",
                        path.file_name().unwrap().to_string_lossy()
                    );
                    fs_paths.insert(rel);
                }
            }
        }

        // New files: in filesystem but not in CSV
        let added: Vec<String> = fs_paths.difference(&csv_paths).cloned().collect();
        // Removed files: in CSV but not in filesystem
        let removed: Vec<String> = csv_paths.difference(&fs_paths).cloned().collect();

        let added_count = added.len();
        let removed_count = removed.len();

        for path in &added {
            csv_registry::add_note_path(&self.config.note_paths_file, path)?;
        }
        for path in &removed {
            csv_registry::remove_note_path(&self.config.note_paths_file, path)?;
        }

        // Rebuild index if there were changes
        if added_count > 0 || removed_count > 0 {
            self.build_index()?;
        }

        Ok((added_count, removed_count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_adds_new_files() {
        let dir = tempfile::tempdir().unwrap();
        let vault_path = dir.path().join("test-vault");
        fs::create_dir_all(&vault_path).unwrap();
        let mut vault = Vault::init(&vault_path).unwrap();

        // Manually create a .typ file without going through new_note
        fs::write(
            vault_path.join("notes/manual.typ"),
            r#"#import "../vault.typ": *
#show: note.with(id: "manual", title: "Manual")
= Manual
"#,
        )
        .unwrap();

        let (added, removed) = vault.sync().unwrap();
        assert_eq!(added, 1);
        assert_eq!(removed, 0);

        let paths = vault.note_paths().unwrap();
        assert!(paths.contains(&"notes/manual.typ".to_string()));
    }

    #[test]
    fn test_sync_removes_missing_files() {
        let dir = tempfile::tempdir().unwrap();
        let vault_path = dir.path().join("test-vault");
        fs::create_dir_all(&vault_path).unwrap();
        let mut vault = Vault::init(&vault_path).unwrap();

        // Delete welcome.typ without updating CSV
        fs::remove_file(vault_path.join("notes/welcome.typ")).unwrap();

        let (added, removed) = vault.sync().unwrap();
        assert_eq!(added, 0);
        assert_eq!(removed, 1);

        let paths = vault.note_paths().unwrap();
        assert!(!paths.contains(&"notes/welcome.typ".to_string()));
    }

    #[test]
    fn test_sync_no_changes() {
        let dir = tempfile::tempdir().unwrap();
        let vault_path = dir.path().join("test-vault");
        fs::create_dir_all(&vault_path).unwrap();
        let mut vault = Vault::init(&vault_path).unwrap();

        let (added, removed) = vault.sync().unwrap();
        assert_eq!(added, 0);
        assert_eq!(removed, 0);
    }
}
