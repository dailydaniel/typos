use notes_core::error::NotesError;
use notes_core::vault::Vault;
use std::env;
use std::path::Path;

/// Discover vault from current directory.
fn open_vault() -> Result<Vault, NotesError> {
    let cwd = env::current_dir().map_err(NotesError::Io)?;
    let root = Vault::discover(&cwd)?;
    Vault::open(&root)
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
    id: Option<&str>,
    parent: Option<&str>,
    tags: &[String],
) -> Result<(), NotesError> {
    let vault = open_vault()?;
    let tag_refs: Vec<&str> = tags.iter().map(|s| s.as_str()).collect();
    let meta = vault.new_note(title, note_type, id, parent, &tag_refs, &[])?;
    println!("Created {} \"{}\" at {}", meta.note_type, meta.title, meta.path);
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

pub fn compile(_file: &str, _format: &str, _output: Option<&str>) -> Result<(), NotesError> {
    // Phase 5: requires typst crate + World trait
    eprintln!("Compile is not yet implemented (Phase 5)");
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
        let tags = if note.tags.is_empty() {
            String::new()
        } else {
            format!(" [{}]", note.tags.join(", "))
        };
        println!("  {} — {}{}", note.id, note.title, tags);
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
                "{:<20} {:<30} {:<8} {}",
                "ID", "TITLE", "TYPE", "TAGS"
            );
            println!("{}", "-".repeat(75));
            for note in &notes {
                let tags = note.tags.join(", ");
                let title = if note.title.len() > 28 {
                    format!("{}...", &note.title[..25])
                } else {
                    note.title.clone()
                };
                println!("{:<20} {:<30} {:<8} {}", note.id, title, note.note_type, tags);
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
