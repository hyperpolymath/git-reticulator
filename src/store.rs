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

/// File-backed store: a versioned JSON envelope on local disk. This is the
/// default persistence for the dogfood loop (`reticulate build` → file →
/// `reticulate query`) — no database required. VeriSimDB remains the intended
/// database of record for the full neuro-symbolic stack; this store exists so
/// the build→query loop works standalone today.
pub mod file {
    use crate::lattice::Lattice;
    use serde::{Deserialize, Serialize};
    use std::fs;
    use std::path::{Path, PathBuf};

    /// Bumped whenever the on-disk shape changes incompatibly.
    pub const FORMAT_VERSION: u32 = 1;
    const FORMAT_NAME: &str = "git-reticulator/lattice";

    #[derive(Serialize, Deserialize)]
    struct Envelope {
        format: String,
        version: u32,
        lattice: Lattice,
    }

    /// Errors from loading a lattice file.
    #[derive(Debug)]
    pub enum LoadError {
        Io(std::io::Error),
        Parse(serde_json::Error),
        /// The file parsed but is not a lattice file, or its version is unsupported.
        Format(String),
    }

    impl std::fmt::Display for LoadError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                LoadError::Io(e) => write!(f, "cannot read lattice file: {e}"),
                LoadError::Parse(e) => write!(f, "cannot parse lattice file: {e}"),
                LoadError::Format(msg) => write!(f, "unsupported lattice file: {msg}"),
            }
        }
    }

    /// Persist a lattice as JSON at `path`; `LatticeStore::persist` creates
    /// parent directories as needed and overwrites any existing file.
    #[derive(Debug)]
    pub struct FileStore {
        path: PathBuf,
    }

    impl FileStore {
        pub fn new(path: impl Into<PathBuf>) -> Self {
            Self { path: path.into() }
        }

        pub fn path(&self) -> &Path {
            &self.path
        }

        /// Load a lattice previously written by [`FileStore`].
        pub fn load(path: &Path) -> Result<Lattice, LoadError> {
            let text = fs::read_to_string(path).map_err(LoadError::Io)?;
            let envelope: Envelope = serde_json::from_str(&text).map_err(LoadError::Parse)?;
            if envelope.format != FORMAT_NAME {
                return Err(LoadError::Format(format!(
                    "format is '{}', expected '{FORMAT_NAME}'",
                    envelope.format
                )));
            }
            if envelope.version != FORMAT_VERSION {
                return Err(LoadError::Format(format!(
                    "version {} not supported (this build reads version {FORMAT_VERSION})",
                    envelope.version
                )));
            }
            Ok(envelope.lattice)
        }
    }

    impl super::LatticeStore for FileStore {
        type Error = std::io::Error;

        fn persist(&mut self, lattice: &Lattice) -> Result<usize, Self::Error> {
            if let Some(parent) = self.path.parent() {
                if !parent.as_os_str().is_empty() {
                    fs::create_dir_all(parent)?;
                }
            }
            let envelope = Envelope {
                format: FORMAT_NAME.to_string(),
                version: FORMAT_VERSION,
                lattice: lattice.clone(),
            };
            let json = serde_json::to_string(&envelope)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            fs::write(&self.path, json)?;
            Ok(lattice.len())
        }
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

    #[test]
    fn file_store_round_trips_a_lattice() {
        let mut b = LatticeBuilder::new();
        let m = b.add_keyword("root".into(), "/".into(), SemanticLevel::Module, None);
        let f = b.add_keyword("a.rs".into(), "/a.rs".into(), SemanticLevel::File, Some(m));
        b.add_keyword(
            "login".into(),
            "/a.rs".into(),
            SemanticLevel::Definition,
            Some(f),
        );
        b.add_relationship(f, m, 1.0, "contains".into());
        let lattice = b.build();

        let path = std::env::temp_dir().join(format!(
            "git-reticulator-roundtrip-{}.json",
            std::process::id()
        ));
        let mut store = file::FileStore::new(&path);
        let n = store.persist(&lattice).unwrap();
        assert_eq!(n, 3);

        let loaded = file::FileStore::load(&path).unwrap();
        assert_eq!(loaded.len(), lattice.len());
        assert_eq!(loaded.edges().len(), lattice.edges().len());
        assert_eq!(loaded.node(2).unwrap().name, "login");
        assert_eq!(loaded.node(2).unwrap().parent, Some(1));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn file_store_load_rejects_garbage_and_missing() {
        assert!(matches!(
            file::FileStore::load(std::path::Path::new("/no/such/lattice.json")),
            Err(file::LoadError::Io(_))
        ));
        let path = std::env::temp_dir().join(format!(
            "git-reticulator-garbage-{}.json",
            std::process::id()
        ));
        std::fs::write(
            &path,
            "{\"format\":\"something-else\",\"version\":1,\"lattice\":{\"nodes\":[],\"edges\":[]}}",
        )
        .unwrap();
        assert!(matches!(
            file::FileStore::load(&path),
            Err(file::LoadError::Format(_))
        ));
        let _ = std::fs::remove_file(&path);
    }
}
