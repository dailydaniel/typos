#import "@local/notes:0.1.0": new-vault, as-branch

#let vault = new-vault(
  index: json("notes-index.json"),
)

// Note types
#let tag = (vault.note-type)("tag")
#let note = (vault.note-type)("note", fields: (tags: (), links: ()))
#let task = (vault.note-type)("task", fields: (tags: (), priority: ""))
#let report = (vault.note-type)("report", fields: (tags: ()))

// Cross-references
#let xlink = vault.xlink
