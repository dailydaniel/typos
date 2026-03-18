use crate::error::NotesError;
use crate::vault::Vault;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub node_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
}

impl Vault {
    /// Generate graph data from index.
    pub fn graph_data(&self) -> Result<GraphData, NotesError> {
        let index = self.index.as_ref().ok_or_else(|| {
            NotesError::NoteNotFound(
                "Index not loaded. Call load_index() or build_index() first.".to_string(),
            )
        })?;

        let nodes: Vec<GraphNode> = index
            .notes
            .iter()
            .map(|n| GraphNode {
                id: n.id.clone(),
                label: n.title.clone(),
                node_type: n.note_type.clone(),
            })
            .collect();

        let edges: Vec<GraphEdge> = index
            .links
            .iter()
            .map(|l| GraphEdge {
                source: l.source.clone(),
                target: l.target.clone(),
            })
            .collect();

        Ok(GraphData { nodes, edges })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_graph_data() {
        let dir = tempfile::tempdir().unwrap();
        let vault_path = dir.path().join("test-vault");
        fs::create_dir_all(&vault_path).unwrap();
        let mut vault = Vault::init(&vault_path).unwrap();

        vault
            .new_note("Node A", "note", Some("a"), None, &[], &[])
            .unwrap();
        vault
            .new_note("Node B", "note", Some("b"), None, &[], &[])
            .unwrap();

        // Add link from a to b
        let note_path = vault_path.join("notes/a.typ");
        let content = fs::read_to_string(&note_path).unwrap();
        fs::write(&note_path, format!("{}\n#xlink(\"b\")\n", content)).unwrap();

        vault.build_index().unwrap();
        let graph = vault.graph_data().unwrap();

        assert_eq!(graph.nodes.len(), 3); // welcome + a + b
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].source, "a");
        assert_eq!(graph.edges[0].target, "b");
    }
}
