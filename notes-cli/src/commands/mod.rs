use notes_core::error::NotesError;
use notes_core::types;
use notes_core::vault::Vault;
use std::env;
use std::path::Path;

/// Discover vault from current directory.
fn open_vault() -> Result<Vault, NotesError> {
    let cwd = env::current_dir().map_err(NotesError::Io)?;
    let root = Vault::discover(&cwd)?;
    Vault::open(&root)
}

/// Resolve a note argument to a relative file path.
/// Accepts either a file path ("notes/foo.typ") or an id ("foo", "a/b/c").
fn resolve_note_path(vault: &Vault, input: &str) -> Result<String, NotesError> {
    // If it looks like a file path, use as-is
    if input.ends_with(".typ") {
        return Ok(input.to_string());
    }
    // Otherwise treat as id → convert to path
    let rel_path = types::id_to_path(input);
    let abs_path = vault.config.root.join(&rel_path);
    if abs_path.exists() {
        Ok(rel_path)
    } else {
        Err(NotesError::NoteNotFound(input.to_string()))
    }
}

/// Discover vault and load its index.
fn open_vault_with_index() -> Result<Vault, NotesError> {
    let mut vault = open_vault()?;
    vault.load_index()?;
    Ok(vault)
}

pub fn init(path: &str) -> Result<(), NotesError> {
    let target = if path == "." {
        env::current_dir().map_err(NotesError::Io)?
    } else {
        let p = Path::new(path).to_path_buf();
        std::fs::create_dir_all(&p)?;
        p
    };

    Vault::init(&target)?;
    println!("Initialized vault at {}", target.display());
    Ok(())
}

pub fn new_note(
    title: &str,
    note_type: &str,
) -> Result<(), NotesError> {
    let vault = open_vault()?;
    let meta = vault.new_note(title, note_type, &[])?;
    println!("Created {} \"{}\" at {} (id: {})", meta.note_type, meta.title, meta.path, meta.id);
    Ok(())
}

pub fn index() -> Result<(), NotesError> {
    let mut vault = open_vault()?;
    let count = vault.build_index()?;
    let link_count = vault.index.as_ref().map(|i| i.links.len()).unwrap_or(0);
    println!("Indexed {} notes, {} links", count, link_count);
    Ok(())
}

pub fn sync() -> Result<(), NotesError> {
    let mut vault = open_vault()?;
    let (added, removed) = vault.sync()?;
    println!("Synced: +{} added, -{} removed", added, removed);
    Ok(())
}

pub fn compile(file: &str, format: &str, output: Option<&str>) -> Result<(), NotesError> {
    let mut vault = open_vault()?;
    let rel_path = resolve_note_path(&vault, file)?;
    let note_path = vault.config.root.join(&rel_path);

    let output_path = match output {
        Some(p) => std::path::PathBuf::from(p),
        None => vault.default_output_path(format),
    };

    let reindexed = vault.reindex_if_stale()?;
    if reindexed {
        eprintln!("Index updated.");
    }

    vault.compile_note(&note_path, &output_path, format)?;
    println!("Compiled {} → {}", rel_path, output_path.display());
    Ok(())
}

pub fn watch(file: &str, format: &str) -> Result<(), NotesError> {
    let mut vault = open_vault()?;
    let rel_path = resolve_note_path(&vault, file)?;
    let note_path = vault.config.root.join(&rel_path);

    let output_path = vault.default_output_path(format);
    vault.watch_and_compile(&note_path, &output_path, format)?;
    Ok(())
}

pub fn delete(id: &str) -> Result<(), NotesError> {
    let vault = open_vault()?;
    vault.delete_note(id)?;
    println!("Deleted \"{}\"", id);
    Ok(())
}

pub fn rename(old_id: &str, new_id: &str) -> Result<(), NotesError> {
    let vault = open_vault()?;
    let renamed = vault.rename_note(old_id, new_id)?;
    for id in &renamed {
        println!("  {}", id);
    }
    println!("Renamed {} note(s)", renamed.len());
    Ok(())
}

pub fn search(query: &str, note_type: Option<&str>) -> Result<(), NotesError> {
    let vault = open_vault_with_index()?;
    let results = vault.search(query)?;
    let results: Vec<_> = match note_type {
        Some(t) => results.into_iter().filter(|n| n.note_type == t).collect(),
        None => results,
    };

    if results.is_empty() {
        println!("No results for \"{}\"", query);
        return Ok(());
    }

    for note in &results {
        println!("  {} — {} ({})", note.id, note.title, note.note_type);
    }
    println!("{} result(s)", results.len());
    Ok(())
}

pub fn backlinks(id: &str) -> Result<(), NotesError> {
    let vault = open_vault_with_index()?;
    let results = vault.backlinks(id)?;

    if results.is_empty() {
        println!("No backlinks for \"{}\"", id);
        return Ok(());
    }

    println!("Backlinks for \"{}\":", id);
    for note in &results {
        println!("  {} — {} ({})", note.id, note.title, note.note_type);
    }
    Ok(())
}

pub fn list(note_type: Option<&str>, format: &str) -> Result<(), NotesError> {
    let vault = open_vault_with_index()?;
    let notes = vault.list_notes(note_type)?;

    if notes.is_empty() {
        println!("No notes found.");
        return Ok(());
    }

    match format {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&notes).unwrap());
        }
        _ => {
            // Table format
            println!(
                "{:<30} {:<25} {:<8} {}",
                "ID", "TITLE", "TYPE", "PARENT"
            );
            println!("{}", "-".repeat(80));
            for note in &notes {
                let parent = note.parent.as_deref().unwrap_or("");
                let id = if note.id.len() > 28 {
                    format!("{}...", &note.id[..25])
                } else {
                    note.id.clone()
                };
                let title = if note.title.len() > 23 {
                    format!("{}...", &note.title[..20])
                } else {
                    note.title.clone()
                };
                println!("{:<30} {:<25} {:<8} {}", id, title, note.note_type, parent);
            }
            println!("\n{} note(s)", notes.len());
        }
    }
    Ok(())
}

pub fn graph(format: &str, _output: Option<&str>) -> Result<(), NotesError> {
    let vault = open_vault_with_index()?;
    let graph = vault.graph_data()?;

    match format {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&graph).unwrap());
        }
        _ => {
            println!("Graph: {} nodes, {} edges", graph.nodes.len(), graph.edges.len());
            for edge in &graph.edges {
                println!("  {} -> {}", edge.source, edge.target);
            }
        }
    }
    Ok(())
}
