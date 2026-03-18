// Cross-reference links between notes

/// Create an xlink function bound to a specific index.
#let make-xlink(index) = {
  (id, body: none) => {
    let notes = index.at("notes", default: ())
    let target = notes.find(n => n.at("id", default: "") == id)
    if target != none {
      let display = if body == none { target.at("title", default: id) } else { body }
      link(target.at("path", default: ""), display)
    } else {
      text(fill: red)[#id (not found)]
    }
  }
}

// Placeholder xlink — replaced by vault-specific version in user's vault.typ
#let xlink(id, body: none) = {
  text(fill: red)[\##id (no vault loaded)]
}
