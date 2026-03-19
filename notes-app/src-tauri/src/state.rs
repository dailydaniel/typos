use notes_core::vault::Vault;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct AppState {
    pub vault: Mutex<Option<Vault>>,
    /// Resolved path to the bundled typst sidecar binary.
    pub typst_binary: Mutex<Option<PathBuf>>,
    /// Resolved path to the bundled packages directory.
    pub package_path: Mutex<Option<PathBuf>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            vault: Mutex::new(None),
            typst_binary: Mutex::new(None),
            package_path: Mutex::new(None),
        }
    }
}
