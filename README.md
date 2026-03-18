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
# Install the CLI globally (requires Rust toolchain)
cargo install --path notes-cli

# The `notes` command is now available everywhere
notes --help

# Install the Typst framework as a local package
# macOS:
cp -r notes-framework/src/ ~/Library/Application\ Support/typst/packages/local/notes/0.1.0/src/
cp notes-framework/typst.toml ~/Library/Application\ Support/typst/packages/local/notes/0.1.0/
# Linux:
cp -r notes-framework/src/ ~/.local/share/typst/packages/local/notes/0.1.0/src/
cp notes-framework/typst.toml ~/.local/share/typst/packages/local/notes/0.1.0/
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
notes new "Build MVP" --type task
notes new "programming/rust" --type note
notes new "programming/rust/closures" --type card
notes new "programming/python"
```

The path syntax (`/`) creates a hierarchy. Parent notes are auto-created if they don't exist. Each command creates a `.typ` file in `notes/` with `--` as the path separator in the filename:

```
notes/build-mvp.typ                  → id: "build-mvp"
notes/programming.typ                → id: "programming"
notes/programming--rust.typ          → id: "programming/rust"
notes/programming--rust--closures.typ → id: "programming/rust/closures"
```

### Build the index

```bash
notes index
# Indexed 5 notes, 4 links
```

Parses all registered `.typ` files, extracts metadata and `xlink` calls, writes `notes-index.json`.

### Search and query

```bash
notes list
# ID                             TITLE          TYPE     PARENT
# ---------------------------------------------------------------------------
# welcome                        Welcome        note
# build-mvp                      Build MVP      task
# programming                    programming    note
# programming/rust               rust           note     programming
# programming/rust/closures      closures       card     programming/rust
# programming/python             python         note     programming

notes list --type card
notes list --format json

notes search "rust"
#   programming/rust — rust (note)
#   programming/rust/closures — closures (card)
# 2 result(s)

notes backlinks "programming/rust"
# Backlinks for "programming/rust":
#   programming/rust/closures — closures (card)

notes graph
# Graph: 5 nodes, 2 edges
#   programming/rust/closures -> programming/rust

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
notes compile notes/programming--rust--closures.typ              # → assets/current.html
notes compile notes/programming--rust--closures.typ --format pdf  # → assets/current.pdf
notes compile notes/welcome.typ -o out.pdf --format pdf           # → custom path
```

Default output is `assets/current.html`. Before compiling, the index is automatically rebuilt if any `.typ` files have changed.

Each compiled note includes a metadata header block (title, type, id, parent, custom fields) and backlinks at the bottom.

### Watch mode

```bash
notes watch notes/welcome.typ              # → assets/current.html, live reload
notes watch notes/welcome.typ --format pdf  # → assets/current.pdf
```

Watches the vault for `.typ` file changes and recompiles automatically. On each change: reindex if stale → recompile → write output. Useful with Tauri or a browser for live preview.

## Writing Notes

A note is a regular `.typ` file. The title heading is rendered automatically — no need to write `= Title` manually:

```typst
#import "../vault.typ": *

#show: card.with(
  title: "closures",
  tags: ("rust", "fp", "@programming/python"),
  difficulty: "hard",
)

Closures capture variables from their environment.
See also #xlink("programming/rust/traits").
```

**`#show: type.with(title: "...")`** — registers the note with typed metadata. The `id` and `parent` are derived from the filename automatically — you only need to specify `title` and any custom fields.

### Cross-references

There are two ways to link to other notes:

- **In properties** — use `"@id"` string: `tags: ("rust", "@programming/python")`. Rendered as a clickable link in the metadata block, and indexed by `notes index`.
- **In body text** — use `#xlink("id")`: `See #xlink("programming/rust")`. Rendered as an inline link with the target note's title.

Both are indexed as links and appear in backlinks of the target note.

**Backlinks** are rendered automatically at the bottom of each note — no manual setup needed.

**Metadata header** is rendered at the top of each compiled note showing all properties (title, type, id, parent, custom fields). Can be disabled with `show-meta: false` on the note type.

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

- [x] **Compile & watch** — `notes compile` and `notes watch` via typst subprocess
- [ ] **Programmatic compilation** — replace subprocess with `typst` Rust crate (World trait)
- [ ] **Tauri app** — desktop GUI with editor, preview, search (Svelte + Vite)
- [ ] **iOS support** — via Tauri v2 mobile
- [ ] **Graphviz rendering** — `diagraph` integration for visual knowledge graphs
- [ ] **Incremental indexing** — skip unchanged files based on mtime
- [ ] **Parallel parsing** — `rayon` for large vaults
