// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// src/ingest.rs
//
// Repository ingestion. The default path is dependency-free: a std-only
// filesystem walk that turns a directory tree into a Module/File/Definition
// lattice. (Git-history ingestion via `git2` is a future feature seam.)
//
// Every operation is fail-soft: unreadable paths or files yield a smaller
// lattice, never a panic — the resilience tests feed this arbitrary input.

use crate::lattice::{Lattice, LatticeBuilder, NodeId, SemanticLevel};
use std::fs;
use std::path::Path;

/// Maximum directory depth to descend (guards against symlink loops / runaway trees).
const MAX_DEPTH: usize = 64;
/// Maximum number of definition keywords extracted per file.
const MAX_DEFS_PER_FILE: usize = 256;

/// Build a lattice from a filesystem directory tree rooted at `root`.
///
/// Never panics. A non-existent or unreadable `root` yields a one-node lattice
/// (the module root) with no children.
pub fn from_path(root: &str) -> Lattice {
    let mut builder = LatticeBuilder::new();
    let root_path = Path::new(root);
    let root_name = root_path
        .file_name()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(root)
        .to_string();
    let module_id = builder.add_keyword(root_name, root.to_string(), SemanticLevel::Module, None);
    walk(&mut builder, root_path, module_id, 0);
    builder.build()
}

fn walk(builder: &mut LatticeBuilder, dir: &Path, parent: NodeId, depth: usize) {
    if depth >= MAX_DEPTH {
        return;
    }
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return, // fail-soft: unreadable directory
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        // Skip hidden entries (.git, .github, …) — they are not source units.
        if name.starts_with('.') {
            continue;
        }
        if path.is_dir() {
            let sub = builder.add_keyword(
                name,
                path.to_string_lossy().to_string(),
                SemanticLevel::Module,
                Some(parent),
            );
            walk(builder, &path, sub, depth + 1);
        } else if path.is_file() {
            let file_id = builder.add_keyword(
                name,
                path.to_string_lossy().to_string(),
                SemanticLevel::File,
                Some(parent),
            );
            if let Ok(content) = fs::read_to_string(&path) {
                let file_disp = path.to_string_lossy().to_string();
                for def in extract_definitions(&content) {
                    builder.add_keyword(def, file_disp.clone(), SemanticLevel::Definition, Some(file_id));
                }
            }
        }
    }
}

/// Crude but real, language-agnostic definition extraction: capture the
/// identifier following a common definition keyword at the start of a line.
fn extract_definitions(content: &str) -> Vec<String> {
    const KEYWORDS: [&str; 11] = [
        "pub fn ", "fn ", "def ", "class ", "struct ", "enum ", "trait ", "type ", "module ",
        "interface ", "func ",
    ];
    let mut defs = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim_start();
        for kw in KEYWORDS {
            if let Some(rest) = trimmed.strip_prefix(kw) {
                let ident: String = rest
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                if !ident.is_empty() {
                    defs.push(ident);
                }
                break;
            }
        }
        if defs.len() >= MAX_DEFS_PER_FILE {
            break;
        }
    }
    defs.sort();
    defs.dedup();
    defs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nonexistent_path_yields_root_only() {
        let lattice = from_path("/no/such/path/should/exist/12345");
        assert_eq!(lattice.len(), 1); // just the module root
    }

    #[test]
    fn empty_path_does_not_panic() {
        let _ = from_path("");
    }

    #[test]
    fn extracts_definitions_from_source_text() {
        let defs = extract_definitions("pub fn login() {}\nstruct Session;\n// comment\nfn helper() {}");
        assert!(defs.contains(&"login".to_string()));
        assert!(defs.contains(&"Session".to_string()));
        assert!(defs.contains(&"helper".to_string()));
    }
}

// ---------------------------------------------------------------------------
// Git-history ingestion (feature `git-integration`).
//
// Unlike the std-only `from_path` walk, this reads the *committed* HEAD tree
// (tracked files only — never untracked or .gitignore'd), then enriches the
// lattice with temporal coupling: `co_change` relationships between files that
// are repeatedly modified in the same commit. That coupling is a real
// history-derived signal the filesystem walk cannot see.
// ---------------------------------------------------------------------------
#[cfg(feature = "git-integration")]
pub use git_history::from_git;

#[cfg(feature = "git-integration")]
mod git_history {
    use super::{extract_definitions, MAX_DEFS_PER_FILE};
    use crate::lattice::{Lattice, LatticeBuilder, NodeId, SemanticLevel};
    use std::collections::HashMap;

    /// Most recent commits scanned for co-change coupling.
    const MAX_COMMITS: usize = 500;
    /// Commits touching more files than this are treated as bulk/vendoring
    /// changes and excluded from coupling (they create dense spurious edges).
    const MAX_FILES_PER_COMMIT: usize = 25;
    /// Minimum times two files must co-change before an edge is recorded.
    const MIN_COCHANGE: usize = 2;

    /// Build a lattice from a git repository's HEAD tree and commit history.
    ///
    /// Returns an error only if `repo_path` is not a readable git repository or
    /// HEAD is unborn. Every per-commit and per-blob failure is fail-soft
    /// (skipped), so a partially-corrupt history still yields a usable lattice.
    pub fn from_git(repo_path: &str) -> Result<Lattice, git2::Error> {
        let repo = git2::Repository::open(repo_path)?;
        let mut builder = LatticeBuilder::new();

        let root_name = repo
            .workdir()
            .and_then(|w| w.file_name())
            .and_then(|s| s.to_str())
            .map(String::from)
            .unwrap_or_else(|| repo_path.to_string());
        let root_id =
            builder.add_keyword(root_name, repo_path.to_string(), SemanticLevel::Module, None);

        // 1. Structure from the HEAD tree. `dir_ids` is keyed by the
        //    trailing-slash directory path git2 hands the walk callback ("" is
        //    the repo root); `file_ids` by the repo-relative file path, which
        //    matches the paths git diffs report (so step 2 maps cleanly).
        let mut dir_ids: HashMap<String, NodeId> = HashMap::new();
        dir_ids.insert(String::new(), root_id);
        let mut file_ids: HashMap<String, NodeId> = HashMap::new();
        let mut blobs: Vec<(NodeId, git2::Oid, String)> = Vec::new();

        let head = repo.head()?.peel_to_tree()?;
        head.walk(git2::TreeWalkMode::PreOrder, |dir, entry| {
            let name = match entry.name() {
                Some(n) => n.to_string(),
                None => return git2::TreeWalkResult::Ok, // non-UTF-8 path: skip
            };
            let parent = dir_ids.get(dir).copied().unwrap_or(root_id);
            match entry.kind() {
                Some(git2::ObjectType::Tree) => {
                    let full_dir = format!("{dir}{name}/");
                    let id = builder.add_keyword(
                        name,
                        full_dir.trim_end_matches('/').to_string(),
                        SemanticLevel::Module,
                        Some(parent),
                    );
                    dir_ids.insert(full_dir, id);
                }
                Some(git2::ObjectType::Blob) => {
                    let full = format!("{dir}{name}");
                    let fid =
                        builder.add_keyword(name, full.clone(), SemanticLevel::File, Some(parent));
                    file_ids.insert(full.clone(), fid);
                    blobs.push((fid, entry.id(), full)); // read content after the walk
                }
                _ => {}
            }
            git2::TreeWalkResult::Ok
        })?;

        // Definitions, read from blob contents after the walk (keeps the walk
        // closure free of the `repo` borrow).
        for (fid, oid, path) in &blobs {
            if let Ok(blob) = repo.find_blob(*oid) {
                if let Ok(text) = std::str::from_utf8(blob.content()) {
                    for def in extract_definitions(text).into_iter().take(MAX_DEFS_PER_FILE) {
                        builder.add_keyword(def, path.clone(), SemanticLevel::Definition, Some(*fid));
                    }
                }
            }
        }

        // 2. Temporal coupling from history: count file pairs that co-change.
        let mut counts: HashMap<(NodeId, NodeId), usize> = HashMap::new();
        if let Ok(mut revwalk) = repo.revwalk() {
            if revwalk.push_head().is_ok() {
                for oid in revwalk.flatten().take(MAX_COMMITS) {
                    let commit = match repo.find_commit(oid) {
                        Ok(c) => c,
                        Err(_) => continue,
                    };
                    if commit.parent_count() > 1 {
                        continue; // skip merges: their diffs are not real co-edits
                    }
                    let tree = match commit.tree() {
                        Ok(t) => t,
                        Err(_) => continue,
                    };
                    let parent_tree = commit.parent(0).ok().and_then(|p| p.tree().ok());
                    let diff =
                        match repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None) {
                            Ok(d) => d,
                            Err(_) => continue,
                        };
                    let mut changed: Vec<NodeId> = Vec::new();
                    for delta in diff.deltas() {
                        let path = delta.new_file().path().or_else(|| delta.old_file().path());
                        if let Some(p) = path {
                            if let Some(&fid) = file_ids.get(p.to_string_lossy().as_ref()) {
                                changed.push(fid);
                            }
                        }
                    }
                    if changed.len() < 2 || changed.len() > MAX_FILES_PER_COMMIT {
                        continue;
                    }
                    changed.sort_unstable();
                    changed.dedup();
                    for i in 0..changed.len() {
                        for j in (i + 1)..changed.len() {
                            *counts.entry((changed[i], changed[j])).or_insert(0) += 1;
                        }
                    }
                }
            }
        }
        for ((a, b), n) in counts {
            if n >= MIN_COCHANGE {
                builder.add_relationship(a, b, n as f64, "co_change".to_string());
            }
        }

        Ok(builder.build())
    }
}

#[cfg(all(test, feature = "git-integration"))]
mod git_history_tests {
    use super::from_git;

    #[test]
    fn ingests_self_repo_with_structure_and_remains_a_dag() {
        // The package working directory is itself a git repository.
        let Ok(lat) = from_git(".") else {
            panic!("the package working directory should be a git repository");
        };
        assert!(lat.len() > 1, "the HEAD tree should yield more than the root module");
        assert!(
            lat.nodes().iter().any(|k| k.name == "Cargo.toml"),
            "the tracked Cargo.toml should appear as a File node"
        );
        // Adding co_change edges must not break the core invariant (P2a).
        assert!(lat.condense().is_acyclic());
    }
}
