// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// src/query.rs
//
// Token-budgeted context packs over a built lattice. This is the consumer
// surface of the dogfood loop: resolve a keyword to lattice nodes, zoom each
// match to the requested level-of-detail, and render the result within an
// explicit token budget — reporting exactly what was dropped, never silently
// truncating (a truncated pack that looks complete would defeat the point).
//
// Token accounting is the standard chars/4 heuristic. It only has to be
// honest enough to keep a pack near the budget the caller asked for; the
// caller's tokenizer is the ground truth.

use crate::lattice::{Lattice, NodeId, SemanticLevel};
use serde::Serialize;

/// Approximate tokens for a rendered string (chars/4, minimum 1 per line).
fn estimate_tokens(s: &str) -> usize {
    (s.chars().count() / 4).max(1)
}

/// A node as it appears in a context pack.
#[derive(Clone, Debug, Serialize)]
pub struct NodeInfo {
    pub id: NodeId,
    pub name: String,
    pub file: String,
    pub level: String,
}

impl NodeInfo {
    fn from_lattice(lattice: &Lattice, id: NodeId) -> Option<Self> {
        lattice.node(id).map(|k| NodeInfo {
            id: k.id,
            name: k.name.clone(),
            file: k.file.clone(),
            level: k.level.as_str().to_string(),
        })
    }
}

/// One matched node plus its zoomed context.
#[derive(Clone, Debug, Serialize)]
pub struct MatchPack {
    pub node: NodeInfo,
    /// Containment path from the root to the matched node (names, coarse→fine).
    pub path: Vec<String>,
    /// Descendants of the match at the requested level, in lattice order.
    pub descendants: Vec<NodeInfo>,
    /// Descendants that existed but were dropped to stay within budget.
    pub descendants_dropped: usize,
}

/// The full result of a query: every match that fit the budget, plus an
/// honest account of what did not.
#[derive(Clone, Debug, Serialize)]
pub struct QueryResult {
    pub pattern: String,
    pub level: String,
    pub matches: Vec<MatchPack>,
    /// Matched nodes dropped entirely because the budget was exhausted.
    pub matches_dropped: usize,
    /// Estimated tokens of the text rendering of this result.
    pub estimated_tokens: usize,
    pub budget_tokens: usize,
}

/// Case-insensitive substring match on node names. Exact (case-insensitive)
/// matches sort first, then coarser levels before finer, then insertion order —
/// so a module named `auth` beats a definition named `authorize_retry`.
pub fn resolve(lattice: &Lattice, pattern: &str) -> Vec<NodeId> {
    let needle = pattern.to_lowercase();
    let mut hits: Vec<NodeId> = lattice
        .nodes()
        .iter()
        .filter(|k| k.name.to_lowercase().contains(&needle))
        .map(|k| k.id)
        .collect();
    hits.sort_by_key(|&id| {
        let k = &lattice.nodes()[id];
        let exact = k.name.to_lowercase() != needle; // false (exact) sorts first
        (exact, k.level.rank(), id)
    });
    hits
}

/// Containment path from the root to `id` (names, coarse→fine). Guards against
/// malformed parent cycles the same way the lattice core does.
fn containment_path(lattice: &Lattice, id: NodeId) -> Vec<String> {
    let mut path = Vec::new();
    let mut cur = Some(id);
    let mut guard = 0;
    while let Some(x) = cur {
        if guard > lattice.len() {
            break;
        }
        match lattice.node(x) {
            Some(k) => {
                path.push(k.name.clone());
                cur = k.parent;
            }
            None => break,
        }
        guard += 1;
    }
    path.reverse();
    path
}

/// Build a token-budgeted context pack: resolve `pattern`, zoom each match to
/// `level`, and include as much as fits in `budget_tokens` (estimated on the
/// text rendering). Whatever is dropped is counted, never hidden.
pub fn context_pack(
    lattice: &Lattice,
    pattern: &str,
    level: SemanticLevel,
    budget_tokens: usize,
) -> QueryResult {
    let hits = resolve(lattice, pattern);
    let mut result = QueryResult {
        pattern: pattern.to_string(),
        level: level.as_str().to_string(),
        matches: Vec::new(),
        matches_dropped: 0,
        estimated_tokens: 0,
        budget_tokens,
    };

    let mut spent = 0usize;
    for (i, &id) in hits.iter().enumerate() {
        let node = match NodeInfo::from_lattice(lattice, id) {
            Some(n) => n,
            None => continue,
        };
        let path = containment_path(lattice, id);
        let header_cost = estimate_tokens(&format!(
            "## {} ({}) — {}\npath: {}\n",
            node.name,
            node.level,
            node.file,
            path.join(" > ")
        ));
        if spent + header_cost > budget_tokens && !result.matches.is_empty() {
            // No room for even this match's header: drop it and the rest.
            result.matches_dropped = hits.len() - i;
            break;
        }
        spent += header_cost;

        let zoomed = lattice.zoom(id, level);
        let mut descendants = Vec::new();
        let mut dropped = 0usize;
        for &d in &zoomed {
            let info = match NodeInfo::from_lattice(lattice, d) {
                Some(n) => n,
                None => continue,
            };
            let line_cost =
                estimate_tokens(&format!("- {} [{}] {}\n", info.name, info.level, info.file));
            if spent + line_cost > budget_tokens {
                dropped = zoomed.len() - descendants.len();
                break;
            }
            spent += line_cost;
            descendants.push(info);
        }

        result.matches.push(MatchPack {
            node,
            path,
            descendants,
            descendants_dropped: dropped,
        });

        if dropped > 0 {
            // Budget exhausted mid-match: everything after this match is dropped.
            result.matches_dropped = hits.len() - i - 1;
            break;
        }
    }

    result.estimated_tokens = spent;
    result
}

/// Render a [`QueryResult`] as compact, LLM-ready text.
pub fn render_text(result: &QueryResult) -> String {
    let mut out = String::new();
    if result.matches.is_empty() && result.matches_dropped == 0 {
        out.push_str(&format!("no nodes match '{}'\n", result.pattern));
        return out;
    }
    for m in &result.matches {
        out.push_str(&format!(
            "## {} ({}) — {}\npath: {}\n",
            m.node.name,
            m.node.level,
            m.node.file,
            m.path.join(" > ")
        ));
        for d in &m.descendants {
            out.push_str(&format!("- {} [{}] {}\n", d.name, d.level, d.file));
        }
        if m.descendants_dropped > 0 {
            out.push_str(&format!(
                "… {} more {} node(s) omitted (budget)\n",
                m.descendants_dropped, result.level
            ));
        }
    }
    if result.matches_dropped > 0 {
        out.push_str(&format!(
            "… {} more match(es) omitted (budget {} tokens)\n",
            result.matches_dropped, result.budget_tokens
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lattice::LatticeBuilder;

    fn fixture() -> Lattice {
        let mut b = LatticeBuilder::new();
        let m = b.add_keyword("root".into(), "/".into(), SemanticLevel::Module, None);
        let auth = b.add_keyword(
            "auth".into(),
            "/auth".into(),
            SemanticLevel::Module,
            Some(m),
        );
        let f1 = b.add_keyword(
            "auth.rs".into(),
            "/auth/auth.rs".into(),
            SemanticLevel::File,
            Some(auth),
        );
        b.add_keyword(
            "login".into(),
            "/auth/auth.rs".into(),
            SemanticLevel::Definition,
            Some(f1),
        );
        b.add_keyword(
            "authorize_retry".into(),
            "/auth/auth.rs".into(),
            SemanticLevel::Definition,
            Some(f1),
        );
        let f2 = b.add_keyword(
            "db.rs".into(),
            "/db.rs".into(),
            SemanticLevel::File,
            Some(m),
        );
        b.add_keyword(
            "connect".into(),
            "/db.rs".into(),
            SemanticLevel::Definition,
            Some(f2),
        );
        b.build()
    }

    #[test]
    fn resolve_prefers_exact_then_coarse() {
        let lat = fixture();
        let hits = resolve(&lat, "auth");
        // exact module 'auth' (id 1) first, then substring hits
        assert_eq!(hits[0], 1);
        assert!(hits.contains(&2)); // auth.rs
        assert!(hits.contains(&4)); // authorize_retry
    }

    #[test]
    fn resolve_is_case_insensitive_and_misses_cleanly() {
        let lat = fixture();
        assert!(!resolve(&lat, "LOGIN").is_empty());
        assert!(resolve(&lat, "zebra").is_empty());
    }

    #[test]
    fn context_pack_zooms_matches_to_level() {
        let lat = fixture();
        let result = context_pack(&lat, "auth", SemanticLevel::Definition, 10_000);
        assert_eq!(result.matches_dropped, 0);
        let first = &result.matches[0];
        assert_eq!(first.node.name, "auth");
        assert_eq!(first.path, vec!["root", "auth"]);
        let names: Vec<_> = first.descendants.iter().map(|d| d.name.as_str()).collect();
        assert!(names.contains(&"login"));
        assert!(names.contains(&"authorize_retry"));
        assert!(!names.contains(&"connect")); // soundness: other subtree excluded
    }

    #[test]
    fn budget_truncates_and_reports_drops() {
        let lat = fixture();
        let full = context_pack(&lat, "auth", SemanticLevel::Definition, 10_000);
        let full_tokens = full.estimated_tokens;
        assert!(full_tokens > 12);

        let tight = context_pack(&lat, "auth", SemanticLevel::Definition, 12);
        assert!(
            tight.estimated_tokens <= 12,
            "spent {} > budget",
            tight.estimated_tokens
        );
        let dropped_somewhere =
            tight.matches_dropped > 0 || tight.matches.iter().any(|m| m.descendants_dropped > 0);
        assert!(dropped_somewhere, "a tight budget must report drops");
        // ... and the drops are visible in the rendering, not silent.
        let text = render_text(&tight);
        assert!(text.contains("omitted"));
    }

    #[test]
    fn render_text_reports_no_match() {
        let lat = fixture();
        let result = context_pack(&lat, "zebra", SemanticLevel::Definition, 100);
        assert!(render_text(&result).contains("no nodes match"));
    }
}
