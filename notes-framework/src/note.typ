// Note utilities

/// Wrap included content to signal branch mode (not standalone).
/// Usage: #as-branch(include "notes/sub-note.typ")
#let as-branch(body) = {
  metadata((kind: "branch"))
  body
}
