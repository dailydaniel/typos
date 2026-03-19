#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod state;

use state::AppState;
use std::path::PathBuf;

/// Resolve the path to the bundled typst sidecar binary.
/// Tauri places externalBin binaries next to the main executable.
fn resolve_typst_binary() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;

    // Tauri sidecar naming: {name}-{target_triple}
    let target = format!("typst-{}", env!("TARGET_TRIPLE"));
    let with_triple = dir.join(&target);
    if with_triple.exists() {
        return Some(with_triple);
    }

    // Production bundle may strip the triple
    let plain = dir.join("typst");
    if plain.exists() {
        return Some(plain);
    }

    None
}

/// Resolve the path to the bundled packages directory.
/// Tauri places bundle resources in the resource directory.
fn resolve_package_path(app: &tauri::App) -> Option<PathBuf> {
    use tauri::Manager;
    let resource_dir = app.path().resource_dir().ok()?;
    let pkg_dir = resource_dir.join("packages");
    if pkg_dir.exists() {
        Some(pkg_dir)
    } else {
        None
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::new())
        .setup(|app| {
            use tauri::Manager;

            let typst_binary = resolve_typst_binary();
            let package_path = resolve_package_path(app);

            if let Some(ref p) = typst_binary {
                eprintln!("[typst-notes] typst binary: {}", p.display());
            } else {
                eprintln!("[typst-notes] typst binary: using PATH");
            }
            if let Some(ref p) = package_path {
                eprintln!("[typst-notes] package path: {}", p.display());
            }

            let state = app.state::<AppState>();
            *state.typst_binary.lock().unwrap() = typst_binary;
            *state.package_path.lock().unwrap() = package_path;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::open_vault,
            commands::init_vault,
            commands::get_vault_types,
            commands::list_notes,
            commands::create_note,
            commands::delete_note,
            commands::rename_note,
            commands::read_note,
            commands::save_note,
            commands::compile_note,
            commands::compile_note_pdf,
            commands::search_notes,
            commands::get_backlinks,
            commands::get_graph,
            commands::reindex,
            commands::sync_vault,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
