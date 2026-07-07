<!--
SPDX-License-Identifier: CC-BY-SA-4.0
SPDX-FileCopyrightText: 2025-2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
-->

[![OpenSSF Best Practices](https://img.shields.io/badge/OpenSSF-Best_Practices-green?logo=opensourcesecurity)](https://www.bestpractices.dev/en/projects/new?repo_url=https://github.com/hyperpolymath/git-reticulator)
[![License: MPL-2.0](https://img.shields.io/badge/License-MPL--2.0-blue.svg)](https://github.com/hyperpolymath/palimpsest-license)
[![CRG C](https://img.shields.io/badge/CRG-C-yellow?style=flat-square)](https://github.com/hyperpolymath/standards/tree/main/component-readiness-grades)

Semantic-lattice + embedding builder for git repositories — the
**symbolic half of a neuro-symbolic retrieval stack**. It lifts a repo
from raw commits/blobs to a typed, hierarchical, embeddable structure
you can **zoom** into, so an LLM gets the minimal relevant context
instead of the whole tree.

> [!IMPORTANT]
> **Maturity: experimental.** The Rust reference engine is real
> (SCC condensation, partial order, LOD zoom, containment meet) and the
> build→query dogfood loop works end-to-end: `reticulate build` persists
> a lattice file, `reticulate query` returns token-budgeted context
> packs (see `docs/DOGFOOD.adoc`). The AffineScript core in
> `src/lattice/affine/*.affine` is **aspirational** (deferred behind the
> ADR-006 bridge); `postgres`/embeddings are feature-gated **off**;
> mechanized proofs cover abstract order theory only, not yet the Rust
> graph (see <a href="PROOF-NEEDS.md" class="md">PROOF-NEEDS</a>). Read
> `.machine_readable/6a2/STATE.a2ml` for the honest status before
> relying on anything here.

# Why this exists

Existing git-analysis tools work at the raw commit/blob level.
`git-reticulator` lifts the analysis to a **navigable semantic
structure** with two faculties fused at every node: an **order
position** (symbolic) and an **embedding** (neural). That fusion is the
basis for:

- **Token-bounded retrieval** — `zoom_to_node(node,` `level)` returns
  the minimal relevant sub-structure (Level-of-Detail) for an LLM
  prompt.

- **Refactoring-impact / blast-radius** — weighted reachability
  (pgRouting Dijkstra/A\*) over typed edges.

- **Authorship + time queries** — "who owns this concept", "when did it
  enter the codebase" (maps onto verisim’s provenance + temporal
  modalities).

# The neuro-symbolic picture (where this is headed)

     [git-reticulator]      [RAG]            [verisim octad]        [vcl-ut]
      symbolic lattice  ->  embeddings   ->  8-modal substrate  ->  proof-carrying
      + per-node vector     similarity       (graph=lattice,        queries
      (LOD zoom)            search           vector=embeddings,     (FRESHNESS,
                                             provenance=authorship) PROVENANCE…)

The payoff is **proof-carrying retrieval**: neural search **proposes**
context; the symbolic lattice + verisim + vcl-ut **dispose**, so a
retrieved snippet can carry a machine-checked certificate that it really
exists at HEAD (FRESHNESS), was authored by X (PROVENANCE), and isn’t
hallucinated (EXISTENCE). See `.machine_readable/6a2/NEUROSYM.a2ml` and
`.machine_readable/6a2/PLAYBOOK.a2ml`.

# Quickstart

```bash
cargo build --features git-integration   # git-aware ingest (plain `cargo build` = filesystem walk)

# CLI binary is `reticulate` (subcommands: build | query | api):
./target/debug/reticulate build --repo /path/to/repo
#   → ingests the repo, writes /path/to/repo/.git-reticulator/lattice.json

./target/debug/reticulate query --repo /path/to/repo --zoom auth --level definition --budget-tokens 800
#   → token-budgeted context pack (add --format json for machine consumption)

./target/debug/reticulate --help
```

> [!NOTE]
> `build` and `query` are **real** end-to-end (ingest → lattice file →
> budgeted context pack; see `docs/DOGFOOD.adoc`). The `api` server and
> the VeriSimDB store (`--features verisim`) remain thin.

# Architecture

- `src/lib.rs`, `src/cli/main.rs`, `src/api/app.rs` — Rust host (CLI +
  REST shell).

- `src/lattice/affine/*.affine` — intended lattice core (AffineScript;
  aspirational).

- `benches/`, `tests/` — criterion + smoke/contract tests.

- `.machine_readable/6a2/` — canonical project state, ecosystem,
  neuro-symbolic design, playbooks.

# Status & honesty

- **Licence**: MPL-2.0.

- **Maturity**: research / skeleton. API not stable.

- **Formal status**: zero proofs; see
  <a href="PROOF-NEEDS.md" class="md">PROOF-NEEDS</a> for the
  obligations the "lattice" claim incurs.

- **Honest state**: `.machine_readable/6a2/STATE.a2ml`
  (IMPLEMENTED\|SKELETON\|ASPIRATIONAL legend).

# Contributing

See <a href="CONTRIBUTING.md" class="md">CONTRIBUTING</a>. Commits must
be GPG-signed; conventional-commits required (CHANGELOG generated via
`standards` `changelog-reusable.yml`).

# Companion repositories

- [`affinescript`](https://github.com/hyperpolymath/affinescript) — the
  intended core language (compiles to Wasm).

- [`verisimdb`](https://github.com/hyperpolymath/verisimdb) — the octad
  substrate this can feed.

- [`vcl-ut`](https://github.com/hyperpolymath/vcl-ut) — proof-carrying
  query layer over verisim.

- [`standards`](https://github.com/hyperpolymath/standards) — canonical
  estate standards.

- [`k9`](https://github.com/hyperpolymath/k9) — metadata-extraction
  tooling (`k9iser.toml` consumed here).
