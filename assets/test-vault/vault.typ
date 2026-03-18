#import "@local/notes:0.1.0": new-vault, as-branch

#let vault = new-vault(
  index: json("notes-index.json"),
)

// Note types
#let note = (vault.note-type)("note")
#let task = (vault.note-type)("task")
#let card = (vault.note-type)("card")
#let tag = (vault.note-type)("tag")

// Cross-references
#let xlink = vault.xlink
