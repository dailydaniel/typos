// Backlink rendering from index data

/// Render a backlinks section for the given note id.
#let render-backlinks(id, index) = {
  let links = index.at("links", default: ())
  let incoming = links.filter(l => l.at("target", default: "") == id)
  if incoming.len() > 0 {
    let notes = index.at("notes", default: ())
    v(1em)
    line(length: 100%, stroke: 0.5pt + gray)
    text(size: 0.9em, fill: gray)[*Backlinks*]
    for bl in incoming {
      let source = notes.find(n => n.at("id", default: "") == bl.at("source", default: ""))
      if source != none {
        [- #link(source.at("path", default: ""))[#source.at("title", default: "?")] _(#source.at("type", default: "?"))_]
      }
    }
  }
}
