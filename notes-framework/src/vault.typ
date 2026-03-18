// Vault initialization and management

#import "index.typ": read-index, query-index
#import "note-type.typ": make-note-type
#import "xlink.typ": make-xlink
#import "backlinks.typ": render-backlinks
#import "graph.typ": build-graph-from-index

/// Create a new vault object.
///
/// - index: dictionary — pre-loaded index data (from json("notes-index.json"))
/// - formatters: array — optional show-rule functions
///
/// Returns a dictionary with methods:
///   - note-type(name) — create a note type constructor
///   - xlink — cross-reference function
///   - build-graph() — render the knowledge graph
///   - query(type, where, sort-by) — query notes from index
///   - backlinks(id) — render backlinks for a note
///   - index — raw index data
#let new-vault(
  index: (:),
  formatters: (),
) = {
  let idx = read-index(index)
  let note-state = state("notes-current-id", none)

  let _note-type(name) = {
    make-note-type(note-state, name, idx)
  }

  let _xlink = make-xlink(idx)

  let _format(apply-backlinks: true, body) = {
    body
    if apply-backlinks {
      context {
        let current-id = note-state.get()
        if current-id != none {
          render-backlinks(current-id, idx)
        }
      }
    }
  }

  (
    note-type: _note-type,
    xlink: _xlink,
    format: _format,
    build-graph: () => build-graph-from-index(idx),
    query: (type: none, where: none, sort-by: none) => {
      query-index(idx, type: type, where: where, sort-by: sort-by)
    },
    backlinks: (id) => render-backlinks(id, idx),
    index: idx,
  )
}
