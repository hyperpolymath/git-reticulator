// Lattice builder in AffineScript
// Implements Level-of-Detail (LOD) zooming logic to minimize LLM token cost.

import models::{Lattice, Keyword, SemanticLevel, Relationship};

@doc "Builds the semantic lattice of a Git repository."
def build_lattice(repo_path: String, db_uri: String) -> Lattice {
  // 1. Repository Ingestion
  let repo = git2::Repository::open(repo_path).unwrap();
  let files = repo.list_files();

  // 2. Multilevel Keyword Extraction (Hierarchical Zooming)
  // Extract keywords at different LOD levels
  let keywords = files.flat_map(|file| {
    [
      extract_file_level_keyword(file),
      extract_definition_level_keywords(file)
    ].flatten()
  });

  // 3. Topological Relationship Discovery
  // Preserves semantics: "login" -> "session" -> "database"
  let relationships = build_relationships(keywords);

  // 4. Persistence with pgRouting Topography
  let storage = LatticeStorage::new(db_uri);
  storage.store_keywords(keywords);
  storage.store_relationships(relationships);

  return Lattice { keywords, relationships };
}

@doc "Exposes a zoomed-in view of a semantic node to reduce LLM token noise."
def zoom_to_node(node_id: UUID, target_level: SemanticLevel) -> Lattice {
  // Queries pgRouting for children nodes at target_level
  let children = storage.get_children(node_id, target_level);
  let internal_edges = storage.get_edges_between(children);
  
  return Lattice { keywords: children, relationships: internal_edges };
}
