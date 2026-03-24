# Git-Reticulator: Database Setup

To support topological queries and hierarchical LOD (Level of Detail) zooming, Git-Reticulator uses **PostgreSQL** with the **pgRouting** extension.

## 🐘 Prerequisites
- PostgreSQL 14+
- PostGIS
- pgRouting

## 🛠️ Schema Initialization

Run the following SQL to set up the semantic lattice environment:

```sql
-- Enable necessary extensions
CREATE EXTENSION IF NOT EXISTS postgis;
CREATE EXTENSION IF NOT EXISTS pgrouting;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- 1. Keywords Table (Nodes in the Lattice)
-- Stores semantic identities at multiple LOD levels.
CREATE TABLE IF NOT EXISTS keywords (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL,
    file_path TEXT NOT NULL,
    level TEXT NOT NULL CHECK (level IN ('Module', 'File', 'Definition', 'Block')),
    parent_id UUID REFERENCES keywords(id),
    embedding VECTOR(1536), -- Optional: requires pgvector for semantic search
    cluster TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(name, file_path, level)
);

-- Index for hierarchical lookups (Zooming)
CREATE INDEX idx_keywords_parent ON keywords(parent_id);
CREATE INDEX idx_keywords_level ON keywords(level);

-- 2. Relationships Table (Edges in the Lattice)
-- Optimized for pgRouting topological queries.
CREATE TABLE IF NOT EXISTS relationships (
    id SERIAL PRIMARY KEY,
    source_id UUID NOT NULL REFERENCES keywords(id),
    target_id UUID NOT NULL REFERENCES keywords(id),
    weight FLOAT DEFAULT 1.0,
    rel_type TEXT NOT NULL, -- e.g., 'calls', 'contains', 'depends_on'
    
    -- pgRouting required columns for topology
    source INTEGER,
    target INTEGER,
    cost FLOAT DEFAULT 1.0,
    reverse_cost FLOAT DEFAULT -1.0 -- -1 means one-way
);

-- 3. Topology Generation
-- After inserting relationships, run pgr_createTopology to populate source/target integers.
-- This is handled by the Git-Reticulator storage engine.
```

## 🔍 Example pgRouting Query (Shortest Semantic Path)

To find the most direct semantic relationship between two nodes:

```sql
SELECT * FROM pgr_dijkstra(
    'SELECT id, source, target, cost FROM relationships',
    1, -- Start node ID (integer)
    50, -- End node ID (integer)
    directed := true
);
```
