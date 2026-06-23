<!-- SPDX-License-Identifier: MPL-2.0 -->
<!-- Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk> -->

# TOPOLOGY.md — git-reticulator

## Purpose

Semantic-lattice + embedding builder for git repositories. Turns a repo into a
typed, hierarchical, embeddable structure suited to feeding LLMs cheaply and
(eventually) provably — layer 1 of a neuro-symbolic retrieval stack
(see `.machine_readable/6a2/NEUROSYM.a2ml`).

> Status legend: **IMPLEMENTED** (Rust on main) · **SKELETON** (stub/println) ·
> **ASPIRATIONAL** (designed in `.affine`, not compilable yet).

## Module Map (as actually built, 2026-06-03)

```
git-reticulator/
├── src/
│   ├── lib.rs                       # host library — SKELETON (println stubs)
│   ├── cli/main.rs                  # `reticulate` CLI (clap): build|query|api
│   ├── api/app.rs                   # actix-web REST shell — SKELETON (canned JSON)
│   └── lattice/affine/              # ASPIRATIONAL core (AffineScript, cannot compile)
│       ├── models.affine            # Keyword / Relationship / Lattice + SemanticLevel
│       ├── storage.affine           # Postgres + pgRouting persistence
│       └── lattice.affine           # build_lattice / zoom_to_node (LOD)
├── tests/
│   ├── integration_tests.rs         # contract/aspect/e2e (smoke-level)
│   ├── property_tests.rs            # proptest (over stubs)
│   ├── api_tests.rs
│   └── lattice_tests.affine         # ASPIRATIONAL (cannot run)
├── benches/git_reticulator_bench.rs # criterion
├── Cargo.toml                       # Rust package (features: git-integration, db, embeddings…)
└── Justfile                         # task runner
```

There are **no `.as` files** and no `build.sh`; earlier versions of this doc
listed `src/main.as`, `src/git_parser.as`, `src/semantic.as` — those never
existed. The core lives in `src/lattice/affine/*.affine`.

## Data Flow (intended; ASPIRATIONAL end-to-end)

```
[Git repo] ─►(git2)─► [files] ─► [multilevel keyword extraction] ─► [Keyword nodes + embeddings]
                                                                          │
                                                   [typed relationship discovery (calls/contains/…)]
                                                                          │
                                                   [Postgres + pgRouting topology]  ◄── persistence
                                                                          │
                                          [zoom_to_node(level)] ─► token-bounded sub-lattice ─► LLM/RAG
```

Today only the dashed CLI/REST shell exists; every box downstream of `git2` is a
stub or unbuildable. See `.machine_readable/6a2/STATE.a2ml [honest-status]`.

## Key Invariants

- **Honesty**: stubs are documented as stubs; the "lattice" claim is unearned
  until `PROOF-NEEDS.md` P1/P2 are discharged (the structure is a typed digraph
  with cycles — not yet a partial order).
- **LOD is the point**: `zoom_to_node` must return the *minimal* relevant
  sub-lattice (token discipline), and must be sound + complete (PROOF-NEEDS P4).
- **Neuro-symbolic fusion at the node**: every `Keyword` carries both an order
  position and an optional embedding (`models.affine`).
```
