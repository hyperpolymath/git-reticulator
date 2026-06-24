<!--
SPDX-License-Identifier: MPL-2.0
Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
-->
# TEST-NEEDS — git-reticulator

## CRG Grade: C — ACHIEVED 2026-04-04

All required test categories present and passing; `cargo test` green.

> **Update 2026-06-04:** the Rust lattice engine is now real, so the suite is no
> longer smoke-over-stubs. The 10 new lib tests assert genuine lattice
> **properties** (SCC-condensation acyclicity, zoom soundness+completeness,
> meet=LCA, reflexivity) — *tested, not proved* (see PROOF-NEEDS.md "Proof
> status"). The 20 pre-existing integration/property/api tests still pass via the
> IO-free compat shim.

### Test Inventory

| Category | Status | Location | Count |
|---|---|---|---|
| Lattice properties | PASS | `src/lattice/mod.rs` (`#[cfg(test)]`) | 6 |
| Ingest | PASS | `src/ingest.rs` (`#[cfg(test)]`) | 3 |
| Store | PASS | `src/store.rs` (`#[cfg(test)]`) | 1 |
| Property-based | PASS | `tests/property_tests.rs` | 5 |
| E2E / Reflexive | PASS | `tests/integration_tests.rs` | 4 |
| Contract | PASS | `tests/integration_tests.rs` | 3 |
| Aspect | PASS | `tests/integration_tests.rs` | 6 |
| API / CLI | PASS | `tests/api_tests.rs` | 2 |
| Benchmarks (baselined) | PASS | `benches/git_reticulator_bench.rs` | 6 |

Total tests: **30** (`cargo test` exit 0)  
Benchmarks: **6** (Criterion; behavioural criterion benches over fixture repos are owed for the new engine)

### Commands

```sh
# Run all tests
cargo test

# Compile benchmarks (no-run for CI)
cargo bench --no-run

# Run benchmarks (writes HTML reports to target/criterion/)
cargo bench
```

### Next: CRG Grade B

Requires 6 quality targets (linting, formatting, documentation coverage, etc.).
See `.machine_readable/6a2/STATE.a2ml` for details.
