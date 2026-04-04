// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// benches/git_reticulator_bench.rs
// Criterion benchmarks for git-reticulator.
//
// Baselines measured:
//   B1 - build_lattice with a short repo path and db URI
//   B2 - query_lattice with a short zoom node and db URI
//   B3 - build_lattice with a long (4 KiB) repo path
//   B4 - query_lattice with a long (4 KiB) zoom node
//   B5 - Sequential build-then-query pipeline

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use git_reticulator::lattice::affine;

// ---------------------------------------------------------------------------
// B1: build_lattice — short inputs
// ---------------------------------------------------------------------------
fn bench_build_lattice_short(c: &mut Criterion) {
    c.bench_function("build_lattice/short", |b| {
        b.iter(|| {
            affine::build_lattice(
                black_box("github.com/hyperpolymath/git-reticulator"),
                black_box("postgres://localhost:5432/reticulator"),
            )
        })
    });
}

// ---------------------------------------------------------------------------
// B2: query_lattice — short inputs
// ---------------------------------------------------------------------------
fn bench_query_lattice_short(c: &mut Criterion) {
    c.bench_function("query_lattice/short", |b| {
        b.iter(|| {
            affine::query_lattice(
                black_box("module::lattice::affine"),
                black_box("postgres://localhost:5432/reticulator"),
            )
        })
    });
}

// ---------------------------------------------------------------------------
// B3: build_lattice — long inputs (4 KiB)
// ---------------------------------------------------------------------------
fn bench_build_lattice_long(c: &mut Criterion) {
    let long_repo: String = "repo/".repeat(819); // ~4 KiB
    let long_db:   String = "db://".repeat(819);

    c.bench_function("build_lattice/long_4kib", |b| {
        b.iter(|| affine::build_lattice(black_box(&long_repo), black_box(&long_db)))
    });
}

// ---------------------------------------------------------------------------
// B4: query_lattice — long inputs (4 KiB)
// ---------------------------------------------------------------------------
fn bench_query_lattice_long(c: &mut Criterion) {
    let long_node: String = "node::".repeat(682); // ~4 KiB
    let long_db:   String = "db://".repeat(819);

    c.bench_function("query_lattice/long_4kib", |b| {
        b.iter(|| affine::query_lattice(black_box(&long_node), black_box(&long_db)))
    });
}

// ---------------------------------------------------------------------------
// B5: Pipeline — build followed immediately by query
// ---------------------------------------------------------------------------
fn bench_pipeline(c: &mut Criterion) {
    c.bench_function("pipeline/build_then_query", |b| {
        b.iter(|| {
            affine::build_lattice(
                black_box("pipeline-repo"),
                black_box("pipeline://db"),
            );
            affine::query_lattice(
                black_box("pipeline-node"),
                black_box("pipeline://db"),
            );
        })
    });
}

// ---------------------------------------------------------------------------
// B6: Parametric — varying input lengths
// ---------------------------------------------------------------------------
fn bench_build_lattice_parametric(c: &mut Criterion) {
    let mut group = c.benchmark_group("build_lattice/by_input_length");

    for size in [8usize, 64, 256, 1024, 4096] {
        let repo = "x".repeat(size);
        let db   = "d".repeat(size);
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &(repo, db),
            |b, (r, d)| b.iter(|| affine::build_lattice(black_box(r), black_box(d))),
        );
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Criterion entry points
// ---------------------------------------------------------------------------
criterion_group!(
    benches,
    bench_build_lattice_short,
    bench_query_lattice_short,
    bench_build_lattice_long,
    bench_query_lattice_long,
    bench_pipeline,
    bench_build_lattice_parametric,
);
criterion_main!(benches);
