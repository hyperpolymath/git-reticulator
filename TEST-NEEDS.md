# TEST-NEEDS — git-reticulator

## CRG Grade: C — ACHIEVED 2026-04-04

All required test categories for CRG Grade C are present and passing.

> **Honest caveat (2026-06-03):** these 27 tests are *smoke-level* — they call
> `build_lattice`/`query_lattice` (which are `println!` stubs) and assert "does
> not panic". They verify the host compiles and the API surface is reachable;
> they do **not** exercise real lattice logic (there is none yet). The
> property/contract/aspect/e2e labels describe test *categories*, not
> behavioural depth. Behavioural + property tests over a real lattice are owed
> alongside `PROOF-NEEDS.md` P1–P4.

### Test Inventory

| Category | Status | Location | Count |
|---|---|---|---|
| Unit | PASS | `src/lib.rs` (`#[cfg(test)]` `unit_tests` module) | 7 |
| Smoke | PASS | `src/lib.rs` (`unit_tests` module) | 2 |
| Property-based (P2P) | PASS | `tests/property_tests.rs` | 5 |
| E2E / Reflexive | PASS | `tests/integration_tests.rs` | 4 |
| Contract | PASS | `tests/integration_tests.rs` | 3 |
| Aspect | PASS | `tests/integration_tests.rs` | 6 |
| Benchmarks (baselined) | PASS | `benches/git_reticulator_bench.rs` | 6 |

Total tests: **27**  
Benchmarks: **6** (Criterion, compile-verified with `cargo bench --no-run`)

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
