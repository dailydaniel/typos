// Graph visualization from index data

/// Build a text-based graph representation from index.
/// For full Graphviz rendering, use diagraph package.
#let build-graph-from-index(index) = {
  let notes = index.at("notes", default: ())
  let links = index.at("links", default: ())

  if notes.len() == 0 {
    [_No notes in index._]
    return
  }

  [*Notes:* #notes.len() · *Links:* #links.len()]
  v(0.5em)

  for note in notes {
    let id = note.at("id", default: "?")
    let title = note.at("title", default: "?")
    let ntype = note.at("type", default: "?")
    [- *#title* (#ntype) `#id`]
  }

  if links.len() > 0 {
    v(0.5em)
    [*Connections:*]
    for l in links {
      [- `#l.at("source", default: "?")` → `#l.at("target", default: "?")`]
    }
  }
}

/// Generate DOT language string for Graphviz rendering.
/// Use with diagraph: `raw-render(build-dot(index))`
#let build-dot(index) = {
  let notes = index.at("notes", default: ())
  let links = index.at("links", default: ())

  let nodes = notes.map(n => {
    let id = n.at("id", default: "?")
    let title = n.at("title", default: "?")
    let ntype = n.at("type", default: "note")
    let shape = if ntype == "tag" { "diamond" } else if ntype == "task" { "box" } else { "ellipse" }
    "  \"" + id + "\" [label=\"" + title + "\", shape=" + shape + "]"
  }).join("\n")

  let edges = links.map(l => {
    "  \"" + l.at("source", default: "") + "\" -> \"" + l.at("target", default: "") + "\""
  }).join("\n")

  "digraph G {\n  rankdir=LR;\n  node [fontsize=10];\n" + nodes + "\n" + edges + "\n}"
}
