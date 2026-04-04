<!-- SPDX-License-Identifier: PMPL-1.0-or-later -->
<!-- Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk> -->

# TOPOLOGY.md — git-reticulator

## Purpose

Semantic lattice builder for git repositories written in AffineScript. Constructs dependency graphs and semantic relationships from commit histories, code structure, and metadata to enable advanced repository analysis and visualization.

## Module Map

```
git-reticulator/
├── src/
│   ├── main.as                # AffineScript core (compiled to binary)
│   ├── lattice.as             # Lattice construction algorithms
│   ├── git_parser.as          # Git history parsing
│   └── semantic.as            # Semantic relationship inference
├── Cargo.toml                 # Rust package wrapper
├── build.sh                   # Build orchestration
└── examples/
    └── ... (example repositories)
```

## Data Flow

```
[Git Repository] ──► [History Parser] ──► [Commit Graph] ──► [Semantic Lattice]
                                                  ↓
                                          [Dependency Inference]
```

## Key Invariants

- Written in AffineScript for memory-safe concurrent analysis
- Produces semantic lattice that preserves commit relationships
- Can detect unstable or circular dependencies
