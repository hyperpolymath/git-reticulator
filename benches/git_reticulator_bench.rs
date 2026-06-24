// SPDX-License-Identifier: MPL-2.0
// Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// benches/git_reticulator_bench.rs
// Criterion benchmarks for the git-reticulator lattice engine.
//
// These measure the *real* algorithmic core (PROOF-NEEDS P1/P2/P4) rather than
// the IO-free `affine` compat shim, so the numbers are behavioural baselines
// for the algorithms a future AffineScript core must match:
//   engine/condense  - Kosaraju SCC condensation, parametric over lattice size
//   engine/zoom      - LOD descendant selection on a large containment forest
//   engine/meet      - lowest-common-ancestor over deep ancestor chains
//   engine/precedes  - node partial-order query (condenses internally per call)
//   engine/ingest    - std-only filesystem walk of this repo's own `src/` tree

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use git_reticulator::ingest;
use git_reticulator::lattice::{Lattice, LatticeBuilder, SemanticLevel};

/// Build a synthetic lattice of `modules` modules, each with `files_per` files,
/// each with `defs_per` definitions (Module ⊃ File ⊃ Definition). Definitions
/// are wired into a forward call-chain with periodic back-edges, so the SCC
/// condensation has genuine cycles to collapse rather than a trivial DAG.
fn synthetic(modules: usize, files_per: usize, defs_per: usize) -> Lattice {
    let mut b = LatticeBuilder::new();
    let mut defs = Vec::new();
    for mi in 0..modules {
        let m = b.add_keyword(
            format!("mod{mi}"),
            format!("/mod{mi}"),
            SemanticLevel::Module,
            None,
        );
        for fi in 0..files_per {
            let f = b.add_keyword(
                format!("file{mi}_{fi}"),
                format!("/mod{mi}/file{fi}.rs"),
                SemanticLevel::File,
                Some(m),
            );
            for di in 0..defs_per {
                let d = b.add_keyword(
                    format!("def{mi}_{fi}_{di}"),
                    format!("/mod{mi}/file{fi}.rs"),
                    SemanticLevel::Definition,
                    Some(f),
                );
                defs.push(d);
            }
        }
    }
    // forward call chain over every definition
    for w in defs.windows(2) {
        b.add_relationship(w[0], w[1], 1.0, "calls".into());
    }
    // periodic back-edges (every 4th def calls back 3 steps) → real SCCs
    let mut i = 3;
    while i < defs.len() {
        b.add_relationship(defs[i], defs[i - 3], 1.0, "calls".into());
        i += 4;
    }
    b.build()
}

fn bench_condense(c: &mut Criterion) {
    let mut group = c.benchmark_group("engine/condense");
    for &scale in &[1usize, 4, 16, 64] {
        let lat = synthetic(scale, 4, 8); // ~37 nodes per scale unit
        let n = lat.len();
        group.bench_with_input(BenchmarkId::from_parameter(n), &lat, |b, lat| {
            b.iter(|| black_box(lat.condense()))
        });
    }
    group.finish();
}

fn bench_zoom(c: &mut Criterion) {
    let lat = synthetic(16, 8, 16); // ~2.2k nodes, deep containment forest
    c.bench_function("engine/zoom_definitions_from_root", |b| {
        b.iter(|| black_box(lat.zoom(black_box(0), SemanticLevel::Definition)))
    });
}

fn bench_meet(c: &mut Criterion) {
    let lat = synthetic(8, 8, 16);
    let last = lat.len() - 1;
    c.bench_function("engine/meet_across_forest", |b| {
        b.iter(|| black_box(lat.meet(black_box(1), black_box(last))))
    });
}

fn bench_precedes(c: &mut Criterion) {
    // precedes() condenses internally on every call — bench that end-to-end cost.
    let lat = synthetic(4, 4, 8);
    let last = lat.len() - 1;
    c.bench_function("engine/precedes_end_to_end", |b| {
        b.iter(|| black_box(lat.precedes(black_box(0), black_box(last))))
    });
}

fn bench_ingest(c: &mut Criterion) {
    // Real fixture: this repo's own source tree (always present in-tree).
    c.bench_function("engine/ingest_src_tree", |b| {
        b.iter(|| black_box(ingest::from_path(black_box("src"))))
    });
}

criterion_group!(
    benches,
    bench_condense,
    bench_zoom,
    bench_meet,
    bench_precedes,
    bench_ingest,
);
criterion_main!(benches);
