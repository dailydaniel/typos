use crate::error::NotesError;
use std::fs;
use std::path::Path;

/// Read all note paths from CSV (single column, no header).
pub fn read_note_paths(csv_path: &Path) -> Result<Vec<String>, NotesError> {
    if !csv_path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(csv_path)?;
    let paths: Vec<String> = content
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect();
    Ok(paths)
}

/// Append a path to CSV.
pub fn add_note_path(csv_path: &Path, note_path: &str) -> Result<(), NotesError> {
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(csv_path)?;
    writeln!(file, "{}", note_path)?;
    Ok(())
}

/// Remove a path from CSV.
pub fn remove_note_path(csv_path: &Path, note_path: &str) -> Result<(), NotesError> {
    let paths = read_note_paths(csv_path)?;
    let filtered: Vec<&String> = paths.iter().filter(|p| p.as_str() != note_path).collect();
    let content = filtered
        .iter()
        .map(|p| p.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let content = if content.is_empty() {
        String::new()
    } else {
        format!("{}\n", content)
    };
    fs::write(csv_path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_write_csv() {
        let dir = tempfile::tempdir().unwrap();
        let csv = dir.path().join("note-paths.csv");

        // Initially empty
        assert_eq!(read_note_paths(&csv).unwrap(), Vec::<String>::new());

        // Add paths
        add_note_path(&csv, "notes/a.typ").unwrap();
        add_note_path(&csv, "notes/b.typ").unwrap();
        add_note_path(&csv, "notes/c.typ").unwrap();

        let paths = read_note_paths(&csv).unwrap();
        assert_eq!(paths, vec!["notes/a.typ", "notes/b.typ", "notes/c.typ"]);

        // Remove one
        remove_note_path(&csv, "notes/b.typ").unwrap();
        let paths = read_note_paths(&csv).unwrap();
        assert_eq!(paths, vec!["notes/a.typ", "notes/c.typ"]);
    }

    #[test]
    fn test_remove_nonexistent() {
        let dir = tempfile::tempdir().unwrap();
        let csv = dir.path().join("note-paths.csv");
        add_note_path(&csv, "notes/a.typ").unwrap();
        remove_note_path(&csv, "notes/z.typ").unwrap(); // no-op
        let paths = read_note_paths(&csv).unwrap();
        assert_eq!(paths, vec!["notes/a.typ"]);
    }
}
