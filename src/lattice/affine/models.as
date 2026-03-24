// Data models for the Git-Reticulator semantic lattice.
// Supports Hierarchical LOD (Level of Detail) to minimize LLM token cost.

type SemanticLevel = 
  | Module      // Root/Subsystem level
  | File        // File-system object level
  | Definition  // Class/Function/Constant level
  | Block       // Granular logical/control-flow level

type Keyword = {
  name: String,
  file: String,
  level: SemanticLevel,
  parent_id: Option<UUID>,    // For hierarchical "zooming"
  embedding: Option<Vec<f64>>,
  cluster: Option<String>
}

type Relationship = {
  source_id: UUID,
  target_id: UUID,
  weight: f64,
  rel_type: String            // e.g., "calls", "contains", "inherits", "depends_on"
}

type Lattice = {
  keywords: Vec<Keyword>,
  relationships: Vec<Relationship>
}
