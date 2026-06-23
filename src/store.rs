// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// src/store.rs
//
// Persistence seam for built lattices. The lattice is the engine's output;
// where it is stored is a separate concern behind the `LatticeStore` trait.
//
// VeriSimDB is the database of record (user directive): each keyword node maps
// to one octad, the relationships to the graph modality, the embeddings to the
// vector modality, authorship→provenance, history→temporal. The verisim backend
// is feature-gated (`--features verisim`) so the default build stays
// dependency-light and offline. Federation is intentionally NOT wired — a
// single standalone verisim instance covers one repo's lattice; extend to the
// federation coordinator only if a lattice must span multiple stores.

use crate::lattice::Lattice;

/// A backend that can persist a built lattice. `persist` returns the number of
/// nodes the backend accepted.
pub trait LatticeStore {
    type Error: std::fmt::Debug;
    fn persist(&mut self, lattice: &Lattice) -> Result<usize, Self::Error>;
}

/// In-memory store — the default, always-available backend. Useful for tests,
/// dry runs, and as the reference implementation of the trait.
#[derive(Debug, Default)]
pub struct InMemoryStore {
    stored: usize,
}

impl InMemoryStore {
    pub fn new() -> Self {
        Self::default()
    }
    /// Total nodes persisted across all calls.
    pub fn stored(&self) -> usize {
        self.stored
    }
}

impl LatticeStore for InMemoryStore {
    type Error = std::convert::Infallible;
    fn persist(&mut self, lattice: &Lattice) -> Result<usize, Self::Error> {
        self.stored += lattice.len();
        Ok(lattice.len())
    }
}

/// VeriSimDB octad-store backend (feature `verisim`). Talks to `verisim-api`
/// over HTTP; see the module docs for the modality mapping.
#[cfg(feature = "verisim")]
pub mod verisim {
    use crate::lattice::Lattice;

    /// HTTP client for a standalone VeriSimDB instance.
    pub struct VerisimStore {
        base_url: String,
        http: reqwest::Client,
    }

    impl VerisimStore {
        /// `base_url` is the verisim-api root, e.g. `http://localhost:8080`.
        pub fn new(base_url: impl Into<String>) -> Self {
            Self {
                base_url: base_url.into(),
                http: reqwest::Client::new(),
            }
        }

        /// Liveness probe against `GET /health`.
        pub async fn health(&self) -> Result<bool, reqwest::Error> {
            let resp = self
                .http
                .get(format!("{}/health", self.base_url))
                .send()
                .await?;
            Ok(resp.status().is_success())
        }

        /// Persist each lattice node as an octad via `POST /api/v1/octads`,
        /// returning the count the server accepted. Edges are carried in each
        /// octad's graph modality; embeddings in the vector modality.
        pub async fn persist(&self, lattice: &Lattice) -> Result<usize, reqwest::Error> {
            let mut accepted = 0usize;
            for kw in lattice.nodes() {
                let relationships: Vec<_> = lattice
                    .edges()
                    .iter()
                    .filter(|e| e.source == kw.id)
                    .map(|e| {
                        serde_json::json!({
                            "rel_type": e.rel_type,
                            "target": e.target,
                            "weight": e.weight,
                        })
                    })
                    .collect();
                let body = serde_json::json!({
                    "name": kw.name,
                    "document": { "title": kw.name, "body": kw.file },
                    "semantic": { "types": [format!("level:{}", kw.level.as_str())] },
                    "vector": kw.embedding,
                    "graph": { "relationships": relationships },
                });
                let resp = self
                    .http
                    .post(format!("{}/api/v1/octads", self.base_url))
                    .json(&body)
                    .send()
                    .await?;
                if resp.status().is_success() {
                    accepted += 1;
                }
            }
            Ok(accepted)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lattice::{LatticeBuilder, SemanticLevel};

    #[test]
    fn in_memory_store_counts_nodes() {
        let mut b = LatticeBuilder::new();
        b.add_keyword("a".into(), "f".into(), SemanticLevel::Module, None);
        b.add_keyword("b".into(), "f".into(), SemanticLevel::File, Some(0));
        let lattice = b.build();

        let mut store = InMemoryStore::new();
        let n = store.persist(&lattice).unwrap();
        assert_eq!(n, 2);
        assert_eq!(store.stored(), 2);
    }
}
