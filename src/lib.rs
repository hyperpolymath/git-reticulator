// SPDX-License-Identifier: MPL-2.0
// Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// src/lib.rs
//
// git-reticulator — a semantic-lattice engine for git repositories, and the
// symbolic (layer-1) half of a neuro-symbolic retrieval stack. See
// `.machine_readable/6a2/NEUROSYM.a2ml` for the architecture and
// `PROOF-NEEDS.md` for the formal obligations the engine discharges.
//
// Module map:
//   * `lattice` — the pure, dependency-free engine (SCC condensation, partial
//     order, LOD zoom, containment meet). Reference core today; designed to be
//     swapped for an AffineScript→Wasm core later (ADR-001) without host churn.
//   * `ingest`  — repository → lattice (std-only filesystem walk).
//   * `store`   — persistence seam; VeriSimDB octad backend behind `--features verisim`.
//   * `api`     — actix-web REST surface.

#![forbid(unsafe_code)]

pub mod lattice;
pub mod ingest;
pub mod store;

pub mod api {
    pub mod app;
}
