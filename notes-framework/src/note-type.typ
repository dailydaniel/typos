// Note type constructor generation

#import "backlinks.typ": render-backlinks

/// Create a note type constructor function.
/// Returns a function usable with #show: type.with(id: ..., title: ..., ...)
#let make-note-type(note-state, type-name, index) = {
  (id: "", title: "", parent: none, tags: (), created: none, ..extra, body) => {
    assert(id != "", message: "Note id is required")

    // Track current note id via state
    note-state.update(id)

    body

    // Render backlinks at the end
    render-backlinks(id, index)
  }
}
