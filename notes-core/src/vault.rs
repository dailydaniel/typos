use crate::ast;
use crate::csv_registry;
use crate::error::NotesError;
use crate::types::{NotesIndex, VaultConfig, VaultType};
use std::fs;
use std::path::{Path, PathBuf};

/// Represents an open vault
pub struct Vault {
    pub config: VaultConfig,
    pub index: Option<NotesIndex>,
    /// Path to typst binary. If None, uses "typst" from PATH.
    pub typst_binary: Option<PathBuf>,
    /// Path to package directory for --package-path flag.
    pub package_path: Option<PathBuf>,
}

impl Vault {
    /// Create a new vault at the given path.
    pub fn init(path: &Path) -> Result<Vault, NotesError> {
        let root = path.to_path_buf();
        let vault_typ = root.join("vault.typ");

        if vault_typ.exists() {
            return Err(NotesError::VaultAlreadyExists(root));
        }

        // Create directories
        let notes_dir = root.join("notes");
        let assets_dir = root.join("assets");
        fs::create_dir_all(&notes_dir)?;
        fs::create_dir_all(&assets_dir)?;

        // Write vault.typ
        fs::write(
            &vault_typ,
            r#"#import "@local/notes:0.1.0": new-vault, as-branch

#let vault = new-vault(
  index: json("notes-index.json"),
)

// Note types
#let tag = (vault.note-type)("tag")
#let note = (vault.note-type)("note", fields: (tags: (), links: ()))
#let task = (vault.note-type)("task", fields: (tags: (), priority: ""))
#let report = (vault.note-type)("report", fields: (tags: ()))
#let journal = (vault.note-type)("journal", fields: (date: ""))

// Cross-references
#let xlink = vault.xlink
"#,
        )?;

        // Write note-paths.csv with welcome note
        let csv_path = root.join("note-paths.csv");
        csv_registry::add_note_path(&csv_path, "notes/welcome.typ")?;

        // Write welcome note
        fs::write(
            &notes_dir.join("welcome.typ"),
            r#"#import "../vault.typ": *

#show: note.with(
  title: "Welcome",
)

This is your first note. Start writing!
"#,
        )?;

        // Write empty index
        let index = NotesIndex {
            version: 1,
            generated_at: chrono::Utc::now().to_rfc3339(),
            notes: Vec::new(),
            links: Vec::new(),
        };
        let index_path = root.join("notes-index.json");
        fs::write(&index_path, serde_json::to_string_pretty(&index)?)?;

        let config = VaultConfig {
            root: root.clone(),
            note_paths_file: csv_path,
            index_file: index_path,
            notes_dir,
            assets_dir,
        };

        Ok(Vault {
            config,
            index: Some(index),
            typst_binary: None,
            package_path: None,
        })
    }

    /// Open an existing vault.
    pub fn open(path: &Path) -> Result<Vault, NotesError> {
        let root = path.to_path_buf();
        let vault_typ = root.join("vault.typ");

        if !vault_typ.exists() {
            return Err(NotesError::VaultNotFound(root));
        }

        let config = VaultConfig {
            note_paths_file: root.join("note-paths.csv"),
            index_file: root.join("notes-index.json"),
            notes_dir: root.join("notes"),
            assets_dir: root.join("assets"),
            root,
        };

        Ok(Vault {
            config,
            index: None,
            typst_binary: None,
            package_path: None,
        })
    }

    /// Find vault root by walking up directories.
    pub fn discover(path: &Path) -> Result<PathBuf, NotesError> {
        let mut current = path.to_path_buf();
        loop {
            if current.join("vault.typ").exists() {
                return Ok(current);
            }
            if !current.pop() {
                return Err(NotesError::VaultNotFound(path.to_path_buf()));
            }
        }
    }

    /// Load index from notes-index.json.
    pub fn load_index(&mut self) -> Result<&NotesIndex, NotesError> {
        if self.index.is_none() {
            let content = fs::read_to_string(&self.config.index_file)?;
            let index: NotesIndex = serde_json::from_str(&content)?;
            self.index = Some(index);
        }
        Ok(self.index.as_ref().unwrap())
    }

    /// Get note paths from CSV.
    pub fn note_paths(&self) -> Result<Vec<String>, NotesError> {
        csv_registry::read_note_paths(&self.config.note_paths_file)
    }

    /// Extract note type definitions from vault.typ.
    pub fn note_types(&self) -> Result<Vec<VaultType>, NotesError> {
        let vault_typ = self.config.root.join("vault.typ");
        let source = fs::read_to_string(&vault_typ)?;
        Ok(ast::extract_vault_types(&source))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_and_open() {
        let dir = tempfile::tempdir().unwrap();
        let vault_path = dir.path().join("test-vault");
        fs::create_dir_all(&vault_path).unwrap();

        // Init
        let vault = Vault::init(&vault_path).unwrap();
        assert!(vault_path.join("vault.typ").exists());
        assert!(vault_path.join("notes/welcome.typ").exists());
        assert!(vault_path.join("note-paths.csv").exists());
        assert!(vault_path.join("notes-index.json").exists());

        // Check CSV
        let paths = vault.note_paths().unwrap();
        assert_eq!(paths, vec!["notes/welcome.typ"]);

        // Open
        let mut vault2 = Vault::open(&vault_path).unwrap();
        assert!(vault2.index.is_none());
        vault2.load_index().unwrap();
        assert!(vault2.index.is_some());
    }

    #[test]
    fn test_init_already_exists() {
        let dir = tempfile::tempdir().unwrap();
        let vault_path = dir.path().join("test-vault");
        fs::create_dir_all(&vault_path).unwrap();

        Vault::init(&vault_path).unwrap();
        let result = Vault::init(&vault_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_discover() {
        let dir = tempfile::tempdir().unwrap();
        let vault_path = dir.path().join("my-vault");
        fs::create_dir_all(&vault_path).unwrap();
        Vault::init(&vault_path).unwrap();

        let sub = vault_path.join("notes");
        let found = Vault::discover(&sub).unwrap();
        assert_eq!(found, vault_path);
    }

    #[test]
    fn test_discover_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let result = Vault::discover(dir.path());
        assert!(result.is_err());
    }
}
