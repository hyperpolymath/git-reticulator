// SPDX-License-Identifier: MPL-2.0
// Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// tests/property_tests.rs
// Property-based tests for the git-reticulator lattice *engine* (proptest).
//
// The example-based unit tests in `src/lattice/mod.rs` check one fixture; these
// quantify the same laws over many randomly-generated lattices, which is the
// strongest "tested, not proved" evidence we have for the PROOF-NEEDS claims:
//   P2a  condensation is always acyclic (a DAG) for arbitrary digraphs
//   P1a  the derived partial order is reflexive
//   P4   `zoom` is sound AND complete vs. an independent descendant oracle
//   P1b  `meet` is commutative and yields a genuine common ancestor
// Plus robustness: the engine never panics on wild (cyclic-parent) input, and
// the legacy `affine` shim never panics on arbitrary strings.

use git_reticulator::lattice::{affine, Lattice, LatticeBuilder, NodeId, SemanticLevel};
use proptest::prelude::*;
use std::collections::BTreeSet;

fn level_of(idx: u8) -> SemanticLevel {
    match idx % 4 {
        0 => SemanticLevel::Module,
        1 => SemanticLevel::File,
        2 => SemanticLevel::Definition,
        _ => SemanticLevel::Block,
    }
}

// Independent oracle for `zoom`: the transitive containment-descendants of
// `root` whose level matches, derived straight from the parent relation.
fn children_of(lat: &Lattice) -> Vec<Vec<NodeId>> {
    let n = lat.len();
    let mut kids = vec![Vec::new(); n];
    for i in 0..n {
        if let Some(p) = lat.node(i).and_then(|k| k.parent) {
            if p < n {
                kids[p].push(i);
            }
        }
    }
    kids
}

fn brute_descendants(lat: &Lattice, root: NodeId, level: SemanticLevel) -> Vec<NodeId> {
    let kids = children_of(lat);
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    let mut stack: Vec<NodeId> = kids.get(root).cloned().unwrap_or_default();
    while let Some(u) = stack.pop() {
        if !seen.insert(u) {
            continue;
        }
        if let Some(k) = lat.node(u) {
            if k.level == level {
                out.push(u);
            }
        }
        if let Some(grandkids) = kids.get(u) {
            stack.extend(grandkids.iter().copied());
        }
    }
    out.sort_unstable();
    out
}

/// Ancestors of `a` (inclusive), following the parent chain with the same
/// cycle guard the engine uses, so a malformed forest cannot hang the test.
fn ancestors(lat: &Lattice, a: NodeId) -> BTreeSet<NodeId> {
    let mut set = BTreeSet::new();
    let mut cur = Some(a);
    let mut guard = 0;
    while let Some(x) = cur {
        if guard > lat.len() {
            break;
        }
        set.insert(x);
        cur = lat.node(x).and_then(|k| k.parent);
        guard += 1;
    }
    set
}

prop_compose! {
    /// A *valid containment forest*: node `i`'s parent (if any) is strictly less
    /// than `i`, so the parent relation is acyclic and zoom/meet semantics are
    /// well-defined. Call edges are arbitrary within `[0, n)` (self-loops and
    /// cycles allowed — that is what condensation is for).
    fn arb_forest()(n in 1usize..24)
                   (levels in prop::collection::vec(any::<u8>(), n),
                    raw_parents in prop::collection::vec(0usize..=n, n),
                    edges in prop::collection::vec((0usize..n, 0usize..n), 0..=(2 * n)),
                    n in Just(n))
                   -> Lattice {
        let mut b = LatticeBuilder::new();
        for i in 0..n {
            let parent = if raw_parents[i] < i { Some(raw_parents[i]) } else { None };
            b.add_keyword(format!("n{i}"), format!("/n{i}"), level_of(levels[i]), parent);
        }
        for (s, t) in edges {
            b.add_relationship(s, t, 1.0, "calls".into());
        }
        b.build()
    }
}

prop_compose! {
    /// A *wild* lattice: parents may reference any node (including forward refs),
    /// so the parent relation can contain cycles. Exercises the engine's
    /// defensive guards (it must never panic or hang).
    fn arb_wild()(n in 1usize..20)
                 (levels in prop::collection::vec(any::<u8>(), n),
                  parents in prop::collection::vec(prop::option::of(0usize..n), n),
                  edges in prop::collection::vec((0usize..n, 0usize..n), 0..=(2 * n)),
                  n in Just(n))
                 -> Lattice {
        let mut b = LatticeBuilder::new();
        for i in 0..n {
            b.add_keyword(format!("n{i}"), format!("/n{i}"), level_of(levels[i]), parents[i]);
        }
        for (s, t) in edges {
            b.add_relationship(s, t, 1.0, "calls".into());
        }
        b.build()
    }
}

proptest! {
    /// P2 / P2a: SCC condensation is ALWAYS a DAG, and its component map is
    /// well-formed, for arbitrary digraphs.
    #[test]
    fn prop_condensation_always_acyclic(lat in arb_forest()) {
        let cond = lat.condense();
        prop_assert!(cond.is_acyclic(), "condensation must be a DAG");
        prop_assert_eq!(cond.component_of.len(), lat.len());
        prop_assert!(cond.num_components <= lat.len());
        for &comp in &cond.component_of {
            prop_assert!(comp < cond.num_components);
        }
    }

    /// P1a: the derived node partial order is reflexive.
    #[test]
    fn prop_precedes_reflexive(lat in arb_forest()) {
        for i in 0..lat.len() {
            prop_assert!(lat.precedes(i, i), "reflexivity at {}", i);
        }
    }

    /// P4: `zoom` is sound (every result is a genuine same-level descendant of
    /// the root) AND complete (it equals an independent descendant oracle).
    #[test]
    fn prop_zoom_sound_and_complete(lat in arb_forest(), root_raw in 0usize..1000, lvl in any::<u8>()) {
        let root = root_raw % lat.len(); // len >= 1, so always in range
        let level = level_of(lvl);
        let got = lat.zoom(root, level);

        // soundness, checked against the parent relation directly
        for &d in &got {
            if let Some(k) = lat.node(d) {
                prop_assert_eq!(k.level, level, "zoom returned a wrong-level node");
            }
            prop_assert!(ancestors(&lat, d).contains(&root), "zoom result {} is not a descendant of {}", d, root);
        }
        // completeness, against the independent oracle
        prop_assert_eq!(got, brute_descendants(&lat, root, level));
    }

    /// P1b: `meet` is commutative, and any meet is a common ancestor of both
    /// arguments (the honest meet-semilattice fragment).
    #[test]
    fn prop_meet_commutative_and_common_ancestor(lat in arb_forest(), a_raw in 0usize..1000, b_raw in 0usize..1000) {
        let a = a_raw % lat.len();
        let b = b_raw % lat.len();
        prop_assert_eq!(lat.meet(a, b), lat.meet(b, a), "meet must be commutative");
        if let Some(m) = lat.meet(a, b) {
            prop_assert!(ancestors(&lat, a).contains(&m), "meet is not an ancestor of a");
            prop_assert!(ancestors(&lat, b).contains(&m), "meet is not an ancestor of b");
        }
    }

    /// Robustness: the engine never panics on wild input with cyclic parents.
    #[test]
    fn prop_engine_never_panics_on_wild_input(lat in arb_wild(), a_raw in 0usize..1000, b_raw in 0usize..1000) {
        let a = a_raw % lat.len();
        let b = b_raw % lat.len();
        let _ = lat.condense();
        let _ = lat.precedes(a, b);
        let _ = lat.zoom(a, SemanticLevel::Definition);
        let _ = lat.meet(a, b); // exercises the ancestor-chain cycle guard
    }

    /// Legacy `affine` shim: arbitrary strings (incl. empty) never panic.
    #[test]
    fn prop_shim_never_panics(repo in ".*", db in ".*", node in ".*") {
        affine::build_lattice(&repo, &db);
        affine::query_lattice(&node, &db);
    }
}
