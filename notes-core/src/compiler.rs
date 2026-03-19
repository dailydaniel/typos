use crate::error::NotesError;
use crate::vault::Vault;
use std::fs;
use std::path::Path;
use std::process::Command;

impl Vault {
    /// Check if any .typ file is newer than the index and reindex if so.
    pub fn reindex_if_stale(&mut self) -> Result<bool, NotesError> {
        let index_mtime = fs::metadata(&self.config.index_file)
            .and_then(|m| m.modified())
            .ok();

        let paths = self.note_paths()?;
        let needs_reindex = paths.iter().any(|p| {
            let path = self.config.root.join(p);
            let file_mtime = fs::metadata(&path).and_then(|m| m.modified()).ok();
            match (file_mtime, index_mtime) {
                (Some(f), Some(i)) => f > i,
                _ => true,
            }
        });

        if needs_reindex {
            self.build_index()?;
            Ok(true)
        } else {
            self.load_index()?;
            Ok(false)
        }
    }

    /// Compile a note using typst CLI as subprocess.
    /// Auto-reindexes if any files are stale.
    pub fn compile_note(
        &mut self,
        note_path: &Path,
        output: &Path,
        format: &str,
    ) -> Result<(), NotesError> {
        self.reindex_if_stale()?;

        // Ensure output directory exists
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)?;
        }

        let typst_bin = self
            .typst_binary
            .as_deref()
            .unwrap_or(Path::new("typst"));

        let mut cmd = Command::new(typst_bin);
        cmd.arg("compile")
            .arg("--root")
            .arg(&self.config.root)
            .arg(note_path)
            .arg(output);

        if let Some(pkg_path) = &self.package_path {
            cmd.arg("--package-path").arg(pkg_path);
        }

        if format == "html" {
            cmd.arg("--features").arg("html");
        }

        let result = cmd.output().map_err(|e| {
            NotesError::CompileError(format!("Failed to run typst: {e}"))
        })?;

        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            return Err(NotesError::CompileError(stderr.to_string()));
        }

        Ok(())
    }

    /// Get the default output path for a given format.
    pub fn default_output_path(&self, format: &str) -> std::path::PathBuf {
        let ext = match format {
            "pdf" => "pdf",
            _ => "html",
        };
        self.config.assets_dir.join(format!("current.{ext}"))
    }
}
