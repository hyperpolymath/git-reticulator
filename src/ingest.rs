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
