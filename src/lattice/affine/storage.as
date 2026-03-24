// src/lattice/affine/storage.as
// PostgreSQL + pgRouting storage implementation for the semantic lattice.

import models::{Keyword, Relationship, SemanticLevel};

@doc "Persists semantic keywords into the lattice nodes table."
def store_keywords(keywords: Vec<Keyword>, db_uri: String) {
  let conn = postgres::Connection::connect(db_uri).unwrap();
  
  for kw in keywords {
    conn.execute(
      "INSERT INTO keywords (name, file_path, level, parent_id, cluster) 
       VALUES (\$1, \$2, \$3, \$4, \$5)
       ON CONFLICT (name, file_path, level) DO UPDATE 
       SET parent_id = \$4, cluster = \$5",
      &[&kw.name, &kw.file, &kw.level.to_string(), &kw.parent_id, &kw.cluster]
    );
  }
}

@doc "Persists relationships and sets up pgRouting cost weights."
def store_relationships(relationships: Vec<Relationship>, db_uri: String) {
  let conn = postgres::Connection::connect(db_uri).unwrap();
  
  for rel in relationships {
    conn.execute(
      "INSERT INTO relationships (source_id, target_id, weight, rel_type, cost)
       VALUES (\$1, \$2, \$3, \$4, \$3)
       ON CONFLICT DO NOTHING",
      &[&rel.source_id, &rel.target_id, &rel.weight, &rel.rel_type]
    );
  }

  // Finalize pgRouting topology
  // This populates the 'source' and 'target' integer columns 
  // required for Dijkstra/A* pathfinding.
  conn.execute("SELECT pgr_createTopology('relationships', 0.00001)", &[]);
}

@doc "Retrieves child nodes for hierarchical LOD zooming."
def get_children(node_id: UUID, target_level: SemanticLevel, db_uri: String) -> Vec<Keyword> {
  let conn = postgres::Connection::connect(db_uri).unwrap();
  
  return conn.query(
    "SELECT name, file_path, level, parent_id, cluster 
     FROM keywords 
     WHERE parent_id = \$1 AND level = \$2",
    &[&node_id, &target_level.to_string()]
  ).map(|row| row.into());
}
