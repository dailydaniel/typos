# Fix: Pre-Tauri improvements (deferred)

These are known issues and optimizations that don't block Tauri development but should be addressed eventually.

## 1. Index link deduplication

### Problem

If a note has both `@programming/rust` in properties and `#xlink("programming/rust")` in body, the index contains two identical links:

```json
{ "source": "closures", "target": "programming/rust", "source_path": "notes/closures.typ" },
{ "source": "closures", "target": "programming/rust", "source_path": "notes/closures.typ" }
```

### Fix

In `index.rs`, deduplicate links by `(source, target)` pair before writing `notes-index.json`. Simple `Vec::dedup()` after sorting, or use a `HashSet`.

### Impact

Cosmetic. Backlinks count may show inflated numbers. No functional breakage.

## 2. Incremental indexing

### Problem

`build_index()` always parses ALL `.typ` files, even if only one changed. For large vaults (100+ notes) this adds noticeable latency to compile/watch.

### Fix

Compare each file's mtime against a stored mtime in the index (or a separate `.notes-mtimes` cache). Only re-parse files that changed. Merge new metadata into existing index, remove entries for deleted files.

```rust
pub fn incremental_index(&mut self) -> Result<usize, NotesError> {
    let existing = self.load_index()?;
    let paths = self.note_paths()?;
    let mut changed = 0;

    for path in &paths {
        let file_mtime = fs::metadata(path).modified()?;
        let cached_mtime = existing.mtimes.get(path);
        if cached_mtime != Some(&file_mtime) {
            // Re-parse this file, update index entry
            changed += 1;
        }
    }

    // Remove entries for files no longer in paths
    // Write updated index
    Ok(changed)
}
```

### Impact

Performance. Most noticeable in watch mode where reindex happens on every save.

## 3. Parallel parsing with rayon

### Problem

`build_index()` parses files sequentially. For large vaults, this is slow.

### Fix

Add `rayon` dependency. Replace the sequential loop in `index.rs` with `par_iter()`:

```rust
use rayon::prelude::*;

let results: Vec<_> = paths.par_iter()
    .map(|path| {
        let source = fs::read_to_string(path)?;
        ast::extract_from_file(&source, path)
    })
    .collect();
```

### Impact

Performance. Only matters for vaults with 50+ notes. Should be combined with incremental indexing — parallel parse only the changed files.

## 4. Programmatic compilation (World trait)

### Problem

Currently `notes compile` spawns a `typst` subprocess. This works but:
- Requires `typst` CLI installed separately
- No control over Typst's package resolution, fonts, etc.
- Slower startup per compilation (process spawn overhead)
- Can't intercept Typst errors programmatically

### Fix

Implement `typst::World` trait in `notes-core/src/world.rs`. Use `typst` and `typst-kit` crates (v0.14+):

```rust
use typst::World;
use typst_kit::fonts::FontSearcher;
use typst_kit::package::PackageStorage;

struct NotesWorld {
    root: PathBuf,
    main: Source,
    library: LazyHash<Library>,
    fonts: Vec<Font>,
    packages: PackageStorage,
}

impl World for NotesWorld {
    fn library(&self) -> &LazyHash<Library> { &self.library }
    fn main(&self) -> Source { self.main.clone() }
    fn source(&self, id: FileId) -> FileResult<Source> { ... }
    fn file(&self, id: FileId) -> FileResult<Bytes> { ... }
    fn font(&self, index: usize) -> Option<Font> { ... }
    fn today(&self, offset: Option<i64>) -> Option<Datetime> { ... }
}
```

Key challenges:
- Package resolution: `@local/notes:0.1.0` must resolve to the local package
- Font discovery: use `typst-kit::FontSearcher` for system fonts
- HTML export: requires `typst::export::html()` (experimental in 0.14)
- Source mapping: `FileId` ↔ filesystem path mapping

### Impact

Removes `typst` CLI dependency. Faster compilation. Required for iOS/mobile (no subprocess).

### Dependencies

```toml
typst = "0.14"
typst-kit = "0.14"
comemo = "0.4"
```

## Priority

1. **Link deduplication** — trivial, fix anytime
2. **Incremental indexing** — do before vaults grow large
3. **Parallel parsing** — after incremental indexing
4. **World trait** — before mobile/iOS, after Tauri MVP works with subprocess
