use crate::error::NotesError;
use crate::vault::Vault;
use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;

impl Vault {
    /// Watch vault for changes and recompile a target note on each change.
    /// Blocks until interrupted (Ctrl+C).
    pub fn watch_and_compile(
        &mut self,
        note_path: &Path,
        output: &Path,
        format: &str,
    ) -> Result<(), NotesError> {
        // Initial compile
        println!("Compiling {}...", note_path.display());
        self.compile_note(note_path, output, format)?;
        println!("Output: {}", output.display());
        println!("Watching for changes... (Ctrl+C to stop)");

        let (tx, rx) = mpsc::channel::<notify::Result<Event>>();

        let mut watcher = notify::recommended_watcher(tx)
            .map_err(|e| NotesError::CompileError(format!("Watch error: {e}")))?;

        watcher
            .watch(&self.config.notes_dir, RecursiveMode::Recursive)
            .map_err(|e| NotesError::CompileError(format!("Watch error: {e}")))?;

        // Also watch vault.typ for config changes
        let vault_typ = self.config.root.join("vault.typ");
        if vault_typ.exists() {
            let _ = watcher.watch(&vault_typ, RecursiveMode::NonRecursive);
        }

        // Debounce: wait for events to settle
        loop {
            match rx.recv() {
                Ok(Ok(event)) => {
                    if !is_typ_change(&event) {
                        continue;
                    }

                    // Drain additional events within debounce window
                    std::thread::sleep(Duration::from_millis(200));
                    while rx.try_recv().is_ok() {}

                    println!("Change detected, recompiling...");
                    match self.compile_note(note_path, output, format) {
                        Ok(()) => println!("Output: {}", output.display()),
                        Err(e) => eprintln!("Compile error: {e}"),
                    }
                }
                Ok(Err(e)) => {
                    eprintln!("Watch error: {e}");
                }
                Err(_) => {
                    // Channel closed, watcher dropped
                    break;
                }
            }
        }

        Ok(())
    }
}

fn is_typ_change(event: &Event) -> bool {
    matches!(
        event.kind,
        EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
    ) && event
        .paths
        .iter()
        .any(|p| p.extension().is_some_and(|ext| ext == "typ"))
}
