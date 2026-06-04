// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// src/lattice/mod.rs
//
// The git-reticulator lattice engine. This is the pure, dependency-free,
// algorithmic core: it earns the word "lattice" (PROOF-NEEDS.md P1/P2) by
// SCC-condensing the typed relationship digraph into a DAG, deriving a partial
// order from it, and implementing the LOD `zoom` (P4) and containment `meet`.
//
// ARCHITECTURE: this Rust implementation is the reference core today. It sits
// behind the `LatticeCore`-shaped API so a future AffineScript→Wasm core can
// replace it without touching the host (ADR-001, AffineScript-first target).
// All IO (git ingestion, verisim persistence) lives outside this module.

use std::collections::{BTreeSet, HashMap, VecDeque};

/// Index of a node into [`Lattice::nodes`]. Stable for the lattice's lifetime.
pub type NodeId = usize;

/// Level-of-detail tier. The containment hierarchy (`parent`) runs
/// Module ⊃ File ⊃ Definition ⊃ Block.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SemanticLevel {
    Module,
    File,
    Definition,
    Block,
}

impl SemanticLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            SemanticLevel::Module => "module",
            SemanticLevel::File => "file",
            SemanticLevel::Definition => "definition",
            SemanticLevel::Block => "block",
        }
    }

    /// Coarse-to-fine rank (Module = 0 … Block = 3).
    pub fn rank(self) -> u8 {
        match self {
            SemanticLevel::Module => 0,
            SemanticLevel::File => 1,
            SemanticLevel::Definition => 2,
            SemanticLevel::Block => 3,
        }
    }
}

/// A semantic keyword node: simultaneously a lattice element (order position via
/// `parent`/edges) and a neural element (`embedding`) — the neuro-symbolic seam.
#[derive(Clone, Debug)]
pub struct Keyword {
    pub id: NodeId,
    pub name: String,
    pub file: String,
    pub level: SemanticLevel,
    pub parent: Option<NodeId>,
    pub embedding: Option<Vec<f64>>,
    pub cluster: Option<String>,
}

/// A typed, weighted relationship (calls / contains / depends_on / …).
#[derive(Clone, Debug)]
pub struct Relationship {
    pub source: NodeId,
    pub target: NodeId,
    pub weight: f64,
    pub rel_type: String,
}

/// The SCC condensation of a lattice's relationship digraph: the acyclic
/// quotient on which a genuine partial order exists (PROOF-NEEDS P2).
#[derive(Clone, Debug)]
pub struct Condensation {
    /// Component id for each node (parallel to [`Lattice::nodes`]).
    pub component_of: Vec<usize>,
    /// Number of strongly-connected components.
    pub num_components: usize,
    /// DAG adjacency between components (deduplicated, self-loops removed).
    pub dag_adj: Vec<BTreeSet<usize>>,
}

impl Condensation {
    /// Verify acyclicity by Kahn topological sort (PROOF-NEEDS P2a). A genuine
    /// check, not a trust-the-construction assertion: returns false iff a cycle
    /// survived condensation (which must never happen).
    pub fn is_acyclic(&self) -> bool {
        let n = self.num_components;
        let mut indeg = vec![0usize; n];
        for adj in &self.dag_adj {
            for &v in adj {
                indeg[v] += 1;
            }
        }
        let mut queue: VecDeque<usize> = (0..n).filter(|&i| indeg[i] == 0).collect();
        let mut processed = 0;
        while let Some(u) = queue.pop_front() {
            processed += 1;
            for &v in &self.dag_adj[u] {
                indeg[v] -= 1;
                if indeg[v] == 0 {
                    queue.push_back(v);
                }
            }
        }
        processed == n
    }

    /// Set of components reachable from `from` (inclusive).
    pub fn reaches(&self, from: usize) -> BTreeSet<usize> {
        let mut seen = BTreeSet::new();
        let mut stack = vec![from];
        seen.insert(from);
        while let Some(u) = stack.pop() {
            for &v in &self.dag_adj[u] {
                if seen.insert(v) {
                    stack.push(v);
                }
            }
        }
        seen
    }

    /// Partial-order relation on components: `a ≤ b` iff `a` reaches `b`.
    /// Reflexive (a reaches a), antisymmetric (acyclic), transitive.
    pub fn precedes(&self, a: usize, b: usize) -> bool {
        self.reaches(a).contains(&b)
    }
}

/// A semantic lattice: typed keyword nodes + weighted relationships, with the
/// algebra that earns the name (condensation, partial order, meet, LOD zoom).
#[derive(Clone, Debug, Default)]
pub struct Lattice {
    nodes: Vec<Keyword>,
    edges: Vec<Relationship>,
}

impl Lattice {
    pub fn nodes(&self) -> &[Keyword] {
        &self.nodes
    }
    pub fn edges(&self) -> &[Relationship] {
        &self.edges
    }
    pub fn len(&self) -> usize {
        self.nodes.len()
    }
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
    pub fn node(&self, id: NodeId) -> Option<&Keyword> {
        self.nodes.get(id)
    }

    fn adjacency(&self) -> (Vec<Vec<usize>>, Vec<Vec<usize>>) {
        let n = self.nodes.len();
        let mut adj = vec![Vec::new(); n];
        let mut radj = vec![Vec::new(); n];
        for e in &self.edges {
            if e.source < n && e.target < n {
                adj[e.source].push(e.target);
                radj[e.target].push(e.source);
            }
        }
        (adj, radj)
    }

    fn children_map(&self) -> HashMap<NodeId, Vec<NodeId>> {
        let mut map: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
        for kw in &self.nodes {
            if let Some(parent) = kw.parent {
                map.entry(parent).or_default().push(kw.id);
            }
        }
        map
    }

    /// SCC condensation via Kosaraju (iterative — no recursion, so deep/large
    /// graphs cannot overflow the stack). PROOF-NEEDS P2.
    pub fn condense(&self) -> Condensation {
        let n = self.nodes.len();
        let (adj, radj) = self.adjacency();

        // Pass 1: order nodes by DFS finish time (iterative post-order).
        let mut visited = vec![false; n];
        let mut order = Vec::with_capacity(n);
        for start in 0..n {
            if visited[start] {
                continue;
            }
            visited[start] = true;
            let mut stack: Vec<(usize, usize)> = vec![(start, 0)];
            while let Some(&(u, i)) = stack.last() {
                if i < adj[u].len() {
                    // Advance the cursor on the current frame without unwrap():
                    // the frame is guaranteed present (stack.last() just
                    // matched), but we avoid an unguarded panic site (CWE-754).
                    if let Some(top) = stack.last_mut() {
                        top.1 += 1;
                    }
                    let v = adj[u][i];
                    if !visited[v] {
                        visited[v] = true;
                        stack.push((v, 0));
                    }
                } else {
                    order.push(u);
                    stack.pop();
                }
            }
        }

        // Pass 2: assign components on the transpose, in reverse finish order.
        let mut component_of = vec![usize::MAX; n];
        let mut count = 0;
        for &start in order.iter().rev() {
            if component_of[start] != usize::MAX {
                continue;
            }
            component_of[start] = count;
            let mut stack = vec![start];
            while let Some(u) = stack.pop() {
                for &v in &radj[u] {
                    if component_of[v] == usize::MAX {
                        component_of[v] = count;
                        stack.push(v);
                    }
                }
            }
            count += 1;
        }

        // Build the DAG adjacency between components.
        let mut dag_adj = vec![BTreeSet::new(); count];
        for e in &self.edges {
            if e.source < n && e.target < n {
                let (cu, cv) = (component_of[e.source], component_of[e.target]);
                if cu != cv {
                    dag_adj[cu].insert(cv);
                }
            }
        }

        Condensation {
            component_of,
            num_components: count,
            dag_adj,
        }
    }

    /// Node-level partial order derived from the condensation: `a ≤ b`.
    /// PROOF-NEEDS P1a/P3.
    pub fn precedes(&self, a: NodeId, b: NodeId) -> bool {
        if a >= self.nodes.len() || b >= self.nodes.len() {
            return false;
        }
        let cond = self.condense();
        cond.precedes(cond.component_of[a], cond.component_of[b])
    }

    /// LOD zoom: the *exact* set of transitive descendants of `node` (via the
    /// containment hierarchy) whose level equals `level`. Sound (only genuine
    /// descendants) and complete (every such descendant). PROOF-NEEDS P4.
    pub fn zoom(&self, node: NodeId, level: SemanticLevel) -> Vec<NodeId> {
        let children = self.children_map();
        let mut result = Vec::new();
        let mut seen = BTreeSet::new();
        let mut stack: Vec<NodeId> = children.get(&node).cloned().unwrap_or_default();
        while let Some(u) = stack.pop() {
            if !seen.insert(u) {
                continue;
            }
            if self.nodes[u].level == level {
                result.push(u);
            }
            if let Some(kids) = children.get(&u) {
                stack.extend(kids.iter().copied());
            }
        }
        result.sort_unstable();
        result
    }

    fn ancestor_chain(&self, a: NodeId) -> Vec<NodeId> {
        let mut chain = Vec::new();
        let mut cur = Some(a);
        let mut guard = 0;
        while let Some(x) = cur {
            if guard > self.nodes.len() {
                break; // defensive: malformed parent cycle
            }
            chain.push(x);
            cur = self.nodes.get(x).and_then(|k| k.parent);
            guard += 1;
        }
        chain
    }

    /// Meet (greatest lower bound) in the containment forest: the lowest common
    /// ancestor of `a` and `b`. The hierarchy is a meet-semilattice; this is the
    /// honest, provable fragment of PROOF-NEEDS P1b (full lattice join is NOT
    /// claimed — see PROOF-NEEDS.md).
    pub fn meet(&self, a: NodeId, b: NodeId) -> Option<NodeId> {
        if a >= self.nodes.len() || b >= self.nodes.len() {
            return None;
        }
        let chain_a = self.ancestor_chain(a);
        let ancestors_b: BTreeSet<NodeId> = self.ancestor_chain(b).into_iter().collect();
        chain_a.into_iter().find(|x| ancestors_b.contains(x))
    }
}

/// Builder for [`Lattice`]. Node ids are assigned sequentially and equal the
/// insertion index, so relationships can reference them directly.
#[derive(Debug, Default)]
pub struct LatticeBuilder {
    nodes: Vec<Keyword>,
    edges: Vec<Relationship>,
}

impl LatticeBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a keyword node, returning its id.
    pub fn add_keyword(
        &mut self,
        name: String,
        file: String,
        level: SemanticLevel,
        parent: Option<NodeId>,
    ) -> NodeId {
        let id = self.nodes.len();
        self.nodes.push(Keyword {
            id,
            name,
            file,
            level,
            parent,
            embedding: None,
            cluster: None,
        });
        id
    }

    /// Attach an embedding vector to a previously-added node (neuro-symbolic seam).
    pub fn set_embedding(&mut self, id: NodeId, embedding: Vec<f64>) {
        if let Some(node) = self.nodes.get_mut(id) {
            node.embedding = Some(embedding);
        }
    }

    /// Add a typed weighted relationship. Endpoints outside the current node set
    /// are silently ignored at condensation time, so this never panics.
    pub fn add_relationship(
        &mut self,
        source: NodeId,
        target: NodeId,
        weight: f64,
        rel_type: String,
    ) {
        self.edges.push(Relationship {
            source,
            target,
            weight,
            rel_type,
        });
    }

    pub fn build(self) -> Lattice {
        Lattice {
            nodes: self.nodes,
            edges: self.edges,
        }
    }
}

// ---------------------------------------------------------------------------
// Backwards-compatible thin shim (the historical `lattice::affine` surface).
// Deliberately IO-free and fast so the legacy resilience tests (arbitrary /
// huge / injection inputs) stay panic-free and quick. Real ingestion lives in
// `crate::ingest`; real persistence in `crate::store`.
// ---------------------------------------------------------------------------
pub mod affine {
    use super::{LatticeBuilder, SemanticLevel};

    /// Compat entry point. Builds a trivial single-node lattice from `repo` and
    /// reports an engine summary. Never touches the filesystem or network.
    pub fn build_lattice(repo: &str, db: &str) {
        let mut builder = LatticeBuilder::new();
        builder.add_keyword(repo.to_string(), repo.to_string(), SemanticLevel::Module, None);
        let lattice = builder.build();
        let cond = lattice.condense();
        println!(
            "reticulated {repo}: {} node(s), {} component(s), acyclic={} [target: {db}]",
            lattice.len(),
            cond.num_components,
            cond.is_acyclic()
        );
    }

    /// Compat entry point for a zoom request. IO-free.
    pub fn query_lattice(zoom: &str, db: &str) {
        println!("zoom target '{zoom}' [source: {db}] — run `reticulate build` to populate a lattice");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a small fixture: one Module containing two Files, each with a
    /// couple of Definitions, plus a cyclic call relationship.
    fn fixture() -> Lattice {
        let mut b = LatticeBuilder::new();
        let m = b.add_keyword("root".into(), "/".into(), SemanticLevel::Module, None);
        let f1 = b.add_keyword("auth.rs".into(), "/auth.rs".into(), SemanticLevel::File, Some(m));
        let f2 = b.add_keyword("db.rs".into(), "/db.rs".into(), SemanticLevel::File, Some(m));
        let d1 = b.add_keyword("login".into(), "/auth.rs".into(), SemanticLevel::Definition, Some(f1));
        let d2 = b.add_keyword("session".into(), "/auth.rs".into(), SemanticLevel::Definition, Some(f1));
        let d3 = b.add_keyword("connect".into(), "/db.rs".into(), SemanticLevel::Definition, Some(f2));
        // login -> session -> connect -> login  (a cycle, to exercise SCC)
        b.add_relationship(d1, d2, 1.0, "calls".into());
        b.add_relationship(d2, d3, 1.0, "calls".into());
        b.add_relationship(d3, d1, 1.0, "calls".into());
        b.build()
    }

    #[test]
    fn condensation_is_acyclic_even_with_cycles() {
        let lat = fixture();
        let cond = lat.condense();
        assert!(cond.is_acyclic(), "condensation must always be a DAG");
        // the 3-node call cycle collapses to a single component
        assert!(cond.num_components < lat.len());
    }

    #[test]
    fn cycle_nodes_share_a_component() {
        let lat = fixture();
        let cond = lat.condense();
        // login(3), session(4), connect(5) form the cycle
        assert_eq!(cond.component_of[3], cond.component_of[4]);
        assert_eq!(cond.component_of[4], cond.component_of[5]);
    }

    #[test]
    fn precedes_is_reflexive() {
        let lat = fixture();
        for i in 0..lat.len() {
            assert!(lat.precedes(i, i), "reflexivity at {i}");
        }
    }

    #[test]
    fn zoom_is_sound_and_complete() {
        let lat = fixture();
        // Definitions reachable from the root module = all three.
        let defs = lat.zoom(0, SemanticLevel::Definition);
        assert_eq!(defs, vec![3, 4, 5]);
        // Files reachable from the root module = the two files.
        let files = lat.zoom(0, SemanticLevel::File);
        assert_eq!(files, vec![1, 2]);
        // Definitions reachable from file f1 (id 1) = its two definitions only
        // (soundness: connect, in the other file, is excluded).
        let f1_defs = lat.zoom(1, SemanticLevel::Definition);
        assert_eq!(f1_defs, vec![3, 4]);
    }

    #[test]
    fn meet_is_lowest_common_ancestor() {
        let lat = fixture();
        // meet(login, session) = their file (f1, id 1)
        assert_eq!(lat.meet(3, 4), Some(1));
        // meet(login, connect) = the root module (id 0)
        assert_eq!(lat.meet(3, 5), Some(0));
        // idempotent
        assert_eq!(lat.meet(3, 3), Some(3));
        // commutative
        assert_eq!(lat.meet(3, 5), lat.meet(5, 3));
    }

    #[test]
    fn empty_lattice_is_well_formed() {
        let lat = Lattice::default();
        let cond = lat.condense();
        assert!(cond.is_acyclic());
        assert_eq!(cond.num_components, 0);
        assert!(lat.zoom(0, SemanticLevel::File).is_empty());
        assert_eq!(lat.meet(0, 1), None);
    }
}
