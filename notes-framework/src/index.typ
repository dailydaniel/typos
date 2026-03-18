// Index data access

/// Normalize index data (ensure required fields exist).
#let read-index(data) = {
  let result = data
  if "notes" not in result { result.insert("notes", ()) }
  if "links" not in result { result.insert("links", ()) }
  result
}

/// Query notes from the index with optional filters.
#let query-index(index, type: none, where: none, sort-by: none) = {
  let results = index.at("notes", default: ())
  if type != none {
    results = results.filter(n => n.at("type", default: "") == type)
  }
  if where != none {
    results = results.filter(where)
  }
  if sort-by != none {
    results = results.sorted(key: n => n.at(sort-by, default: ""))
  }
  results
}
