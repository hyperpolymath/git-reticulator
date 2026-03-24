// tests/lattice_tests.as
// Test suite for the Git-Reticulator semantic lattice builder.

import models::{SemanticLevel, Keyword, Lattice};
import lattice::{build_lattice, zoom_to_node};

@test "Lattice hierarchy should preserve parent-child relationships" {
  let db_uri = "postgresql://localhost/test_db";
  let lattice = build_lattice("./tests/fixtures/sample_repo", db_uri);
  
  let auth_file = lattice.keywords.find(|k| k.name == "auth.res" && k.level == SemanticLevel::File).unwrap();
  let auth_func = lattice.keywords.find(|k| k.name == "login" && k.level == SemanticLevel::Definition).unwrap();
  
  assert_eq!(auth_func.parent_id, Some(auth_file.id));
}

@test "Zooming into a node should only return relevant LOD sub-nodes" {
  let db_uri = "postgresql://localhost/test_db";
  let zoomed = zoom_to_node(module_id, SemanticLevel::File);
  
  assert!(zoomed.keywords.all(|k| k.level == SemanticLevel::File));
}
