use crate::error::NotesError;
use crate::types::NoteMetadata;
use typst_syntax::{ast, SyntaxNode};

/// Result of parsing a single .typ file
#[derive(Debug)]
pub struct AstExtraction {
    pub metadata: Option<NoteMetadata>,
    pub links: Vec<String>,
}

/// Parse a .typ file and extract metadata + links from its AST.
pub fn extract_from_file(source: &str, file_path: &str) -> Result<AstExtraction, NotesError> {
    let root = typst_syntax::parse(source);
    let mut metadata = None;
    let mut links = Vec::new();
    walk_node(&root, &mut metadata, &mut links, file_path);
    Ok(AstExtraction { metadata, links })
}

fn walk_node(
    node: &SyntaxNode,
    metadata: &mut Option<NoteMetadata>,
    links: &mut Vec<String>,
    file_path: &str,
) {
    // Try to match #show: type.with(...)
    if let Some(show_rule) = node.cast::<ast::ShowRule>() {
        if show_rule.selector().is_none() {
            let transform = show_rule.transform();
            if let ast::Expr::FuncCall(call) = transform {
                if let Some(meta) = extract_note_constructor(call, file_path) {
                    *metadata = Some(meta);
                }
            }
        }
    }

    // Try to match #xlink(...) or #xlink(id: "...")
    if let Some(func_call) = node.cast::<ast::FuncCall>() {
        if let Some(target_id) = extract_xlink(func_call) {
            links.push(target_id);
        }
    }

    // Recurse into children
    for child in node.children() {
        walk_node(child, metadata, links, file_path);
    }
}

/// Extract NoteMetadata from a `type.with(id: "...", title: "...", ...)` call.
fn extract_note_constructor(call: ast::FuncCall, file_path: &str) -> Option<NoteMetadata> {
    let callee = call.callee();
    let ast::Expr::FieldAccess(fa) = callee else {
        return None;
    };
    if fa.field().as_str() != "with" {
        return None;
    }
    let ast::Expr::Ident(type_ident) = fa.target() else {
        return None;
    };
    let type_name = type_ident.as_str().to_string();

    let mut id = None;
    let mut title = None;
    let mut parent = None;
    let mut tags = Vec::new();
    let mut created = None;
    let mut extra = serde_json::Map::new();

    for arg in call.args().items() {
        let ast::Arg::Named(named) = arg else {
            continue;
        };
        let key = named.name().as_str();
        match key {
            "id" => id = expr_to_string(named.expr()),
            "title" => title = expr_to_string(named.expr()),
            "parent" => parent = expr_to_string(named.expr()),
            "tags" => tags = expr_to_string_array(named.expr()),
            "created" => created = expr_to_string(named.expr()),
            _ => {
                if let Some(val) = expr_to_json_value(named.expr()) {
                    extra.insert(key.to_string(), val);
                }
            }
        }
    }

    Some(NoteMetadata {
        id: id?,
        title: title.unwrap_or_default(),
        note_type: type_name,
        parent,
        tags,
        created,
        path: file_path.to_string(),
        extra,
    })
}

/// Extract target id from xlink("id") or xlink(id: "id").
fn extract_xlink(call: ast::FuncCall) -> Option<String> {
    let ast::Expr::Ident(ident) = call.callee() else {
        return None;
    };
    if ident.as_str() != "xlink" {
        return None;
    }

    for arg in call.args().items() {
        match arg {
            // xlink("note-id") — positional
            ast::Arg::Pos(expr) => {
                if let Some(s) = expr_to_string(expr) {
                    return Some(s);
                }
            }
            // xlink(id: "note-id") — named
            ast::Arg::Named(named) => {
                if named.name().as_str() == "id" {
                    return expr_to_string(named.expr());
                }
            }
            _ => {}
        }
    }
    None
}

/// Extract a string literal from an AST expression.
fn expr_to_string(expr: ast::Expr) -> Option<String> {
    match expr {
        ast::Expr::Str(s) => Some(s.get().to_string()),
        _ => None,
    }
}

/// Extract an array of string literals from an AST expression.
fn expr_to_string_array(expr: ast::Expr) -> Vec<String> {
    let ast::Expr::Array(arr) = expr else {
        // Single string treated as one-element array
        if let Some(s) = expr_to_string(expr) {
            return vec![s];
        }
        return Vec::new();
    };
    arr.items()
        .filter_map(|item| match item {
            ast::ArrayItem::Pos(e) => expr_to_string(e),
            _ => None,
        })
        .collect()
}

/// Convert an AST expression to a serde_json::Value (for extra fields).
fn expr_to_json_value(expr: ast::Expr) -> Option<serde_json::Value> {
    match expr {
        ast::Expr::Str(s) => Some(serde_json::Value::String(s.get().to_string())),
        ast::Expr::Int(i) => Some(serde_json::Value::Number(i.get().into())),
        ast::Expr::Bool(b) => Some(serde_json::Value::Bool(b.get())),
        ast::Expr::Array(arr) => {
            let items: Vec<serde_json::Value> = arr
                .items()
                .filter_map(|item| match item {
                    ast::ArrayItem::Pos(e) => expr_to_json_value(e),
                    _ => None,
                })
                .collect();
            Some(serde_json::Value::Array(items))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_show_rule_with_metadata() {
        let source = r#"
#import "vault.typ": *

#show: task.with(
  id: "task-001",
  title: "Build MVP",
  tags: ("dev", "mvp"),
  parent: "project-root",
)

= Build MVP

Some content here.
"#;
        let result = extract_from_file(source, "notes/task-001.typ").unwrap();
        let meta = result.metadata.unwrap();
        assert_eq!(meta.id, "task-001");
        assert_eq!(meta.title, "Build MVP");
        assert_eq!(meta.note_type, "task");
        assert_eq!(meta.parent, Some("project-root".to_string()));
        assert_eq!(meta.tags, vec!["dev", "mvp"]);
        assert_eq!(meta.path, "notes/task-001.typ");
    }

    #[test]
    fn test_extract_simple_note() {
        let source = r#"
#import "vault.typ": *

#show: note.with(
  id: "my-note",
  title: "My Note",
)

= My Note

Hello world.
"#;
        let result = extract_from_file(source, "notes/my-note.typ").unwrap();
        let meta = result.metadata.unwrap();
        assert_eq!(meta.id, "my-note");
        assert_eq!(meta.title, "My Note");
        assert_eq!(meta.note_type, "note");
        assert!(meta.parent.is_none());
        assert!(meta.tags.is_empty());
    }

    #[test]
    fn test_extract_xlinks_named() {
        let source = r#"
#import "vault.typ": *

#show: note.with(id: "a", title: "Note A")

See #xlink(id: "note-b") and #xlink(id: "note-c").
"#;
        let result = extract_from_file(source, "notes/a.typ").unwrap();
        assert_eq!(result.links, vec!["note-b", "note-c"]);
    }

    #[test]
    fn test_extract_xlinks_positional() {
        let source = r#"
#import "vault.typ": *

#show: note.with(id: "a", title: "Note A")

See #xlink("note-b") and also #xlink("note-c").
"#;
        let result = extract_from_file(source, "notes/a.typ").unwrap();
        assert_eq!(result.links, vec!["note-b", "note-c"]);
    }

    #[test]
    fn test_no_show_rule() {
        let source = r#"
= Just a heading

Some content without show rule.
"#;
        let result = extract_from_file(source, "notes/plain.typ").unwrap();
        assert!(result.metadata.is_none());
        assert!(result.links.is_empty());
    }

    #[test]
    fn test_extra_fields() {
        let source = r#"
#import "vault.typ": *

#show: card.with(
  id: "card-1",
  title: "Closures",
  difficulty: "hard",
)
"#;
        let result = extract_from_file(source, "notes/card-1.typ").unwrap();
        let meta = result.metadata.unwrap();
        assert_eq!(meta.note_type, "card");
        assert_eq!(
            meta.extra.get("difficulty").and_then(|v| v.as_str()),
            Some("hard")
        );
    }

    #[test]
    fn test_empty_file() {
        let result = extract_from_file("", "notes/empty.typ").unwrap();
        assert!(result.metadata.is_none());
        assert!(result.links.is_empty());
    }

    #[test]
    fn test_created_field() {
        let source = r#"
#show: note.with(
  id: "dated",
  title: "Dated Note",
  created: "2026-03-18",
)
"#;
        let result = extract_from_file(source, "notes/dated.typ").unwrap();
        let meta = result.metadata.unwrap();
        assert_eq!(meta.created, Some("2026-03-18".to_string()));
    }
}
