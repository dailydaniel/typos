# typst-notes

A note-taking system built on [Typst](https://typst.app/) instead of Markdown. Notes are plain `.typ` files with built-in support for typed metadata, cross-references, backlinks, and knowledge graphs — all powered by Typst's own type system.

Instead of reinventing frontmatter parsers, Dataview-style query languages, and custom renderers, typst-notes lets Typst do what it already does: functions, types, and content transformations. The tooling layer (Rust CLI) handles AST extraction and indexing, while the Typst framework handles rendering.

## Architecture

```
typst-notes/
├── notes-framework/    @local/notes Typst package
├── notes-core/         Rust library (AST parsing, indexing)
├── notes-cli/          CLI binary wrapping notes-core
└── notes-app/          Tauri GUI (planned)
```

**Data flow:**
1. You write `.typ` notes with typed constructors and cross-references
2. `notes index` parses all files via `typst-syntax` AST and builds `notes-index.json`
3. When Typst compiles a note, the framework reads the index to resolve links and render backlinks

## Installation

```bash
# Build the CLI
cargo build --release
# Binary is at target/release/notes

# Install the Typst framework as a local package
# macOS:
cp -r notes-framework/ ~/Library/Application\ Support/typst/packages/local/notes/0.1.0/
# Linux:
cp -r notes-framework/ ~/.local/share/typst/packages/local/notes/0.1.0/
```

## CLI Usage

### Create a vault

```bash
notes init my-vault
cd my-vault
```

This generates:
- `vault.typ` — vault configuration with note type definitions
- `note-paths.csv` — registry of all note files
- `notes-index.json` — metadata index (rebuilt by `notes index`)
- `notes/welcome.typ` — your first note

### Create notes

```bash
notes new "Rust Basics" --type note --tags rust,programming
notes new "Build MVP" --type task --tags dev,mvp
notes new "Closures" --type card --parent rust-basics --tags rust
notes new "Rust" --type tag
```

Each command creates a `.typ` file and registers it in `note-paths.csv`. Note types: `note`, `task`, `card`, `tag`.

### Build the index

```bash
notes index
# Indexed 5 notes, 4 links
```

Parses all registered `.typ` files, extracts metadata and `xlink` calls, writes `notes-index.json`.

### Search and query

```bash
notes list
# ID                   TITLE                          TYPE     TAGS
# ---------------------------------------------------------------------------
# welcome              Welcome                        note
# rust-basics          Rust Basics                    note     rust, programming
# build-mvp            Build MVP                      task     dev, mvp
# closures             Closures                       card     rust
# rust                 Rust                           tag

notes list --type task
notes list --format json

notes search "rust"
#   rust-basics — Rust Basics [rust, programming]
#   closures — Closures [rust]
#   rust — Rust
# 3 result(s)

notes backlinks rust
# Backlinks for "rust":
#   rust-basics — Rust Basics (note)
#   closures — Closures (card)

notes graph
# Graph: 5 nodes, 4 edges
#   rust-basics -> rust
#   build-mvp -> welcome
#   closures -> rust-basics
#   closures -> rust

notes graph --format json
```

### Sync after external changes

If files were added or removed outside the CLI (e.g. `git pull`):

```bash
notes sync
# Synced: +2 added, -1 removed
```

Scans `notes/*.typ`, updates `note-paths.csv`, rebuilds the index.

### Compile notes

```bash
typst compile --root . notes/rust-basics.typ
```

Compilation uses the standard `typst` CLI. The `--root .` flag is needed so notes can import `vault.typ` from the vault root.

## Writing Notes

A note is a regular `.typ` file:

```typst
#import "../vault.typ": *

#show: task.with(
  id: "build-mvp",
  title: "Build MVP",
  tags: ("dev", "mvp"),
)

= Build MVP

Implement the core features. See #xlink("rust-basics") for language reference.
```

**`#show: type.with(...)`** — registers the note with typed metadata. The `id` must be a unique string literal (AST extraction only works with literals).

**`#xlink("note-id")`** — cross-reference to another note. Renders as the target note's title (resolved from the index). Shows red "not found" if the target doesn't exist.

**Backlinks** are rendered automatically at the bottom of each note — no manual setup needed.

## Framework

The Typst framework (`@local/notes`) provides:

| Module | Purpose |
|--------|---------|
| `vault.typ` | `new-vault()` — initializes vault object from index data |
| `note-type.typ` | Creates typed constructors for `#show:` rules |
| `xlink.typ` | Cross-reference resolution via index lookup |
| `backlinks.typ` | Renders incoming links at the end of each note |
| `graph.typ` | Text-based graph + DOT output for Graphviz |
| `index.typ` | Index reading and query helpers |

The user's `vault.typ` ties it together:

```typst
#import "@local/notes:0.1.0": new-vault, as-branch

#let vault = new-vault(
  index: json("notes-index.json"),
)

#let note = (vault.note-type)("note")
#let task = (vault.note-type)("task")
#let card = (vault.note-type)("card")
#let tag  = (vault.note-type)("tag")
#let xlink = vault.xlink
```

## Roadmap

- [ ] **Programmatic compilation** — `notes compile` via the `typst` Rust crate (World trait)
- [ ] **Tauri app** — desktop GUI with editor, preview, search (Svelte + Vite)
- [ ] **iOS support** — via Tauri v2 mobile
- [ ] **Watch mode** — `notes watch` for auto-reindexing on file changes
- [ ] **Graphviz rendering** — `diagraph` integration for visual knowledge graphs
- [ ] **Incremental indexing** — skip unchanged files based on mtime
- [ ] **Parallel parsing** — `rayon` for large vaults
