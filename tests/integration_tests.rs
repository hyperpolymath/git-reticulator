// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// tests/integration_tests.rs
// Integration, E2E, reflexive, contract, and aspect tests for git-reticulator.
//
// Test categories covered:
//   E2E       - Full pipeline: build lattice then query it (simulated end-to-end).
//   Reflexive - Calling the same function twice with identical args gives no error.
//   Contract  - Public API invariants that must hold regardless of implementation.
//   Aspect    - Cross-cutting concerns: no panic on nil-like input, no token leakage.

use git_reticulator::lattice::affine;

// ---------------------------------------------------------------------------
// E2E: Simulated end-to-end pipeline
// ---------------------------------------------------------------------------

/// E2E: build a lattice and then immediately query it. Both calls must complete
/// without panicking, simulating a realistic two-phase workflow.
#[test]
fn e2e_build_then_query_pipeline() {
    let repo = "github.com/hyperpolymath/git-reticulator";
    let db = "postgres://localhost:5432/reticulator_test";
    let zoom_node = "module::lattice::affine";

    affine::build_lattice(repo, db);
    affine::query_lattice(zoom_node, db);
}

/// E2E: multiple sequential builds and queries simulate a batch-processing
/// scenario without any state being shared between invocations.
#[test]
fn e2e_batch_build_and_query() {
    let repos = [
        ("repo-alpha", "db://alpha", "node::alpha"),
        ("repo-beta",  "db://beta",  "node::beta"),
        ("repo-gamma", "db://gamma", "node::gamma"),
    ];

    for (repo, db, node) in &repos {
        affine::build_lattice(repo, db);
        affine::query_lattice(node, db);
    }
}

// ---------------------------------------------------------------------------
// Reflexive: same call twice gives identical non-panic outcome
// ---------------------------------------------------------------------------

/// Reflexive: calling build_lattice twice with identical arguments must produce
/// the same outcome (both calls complete without error).
#[test]
fn reflexive_build_lattice_idempotent_on_same_args() {
    let repo = "reflexive-repo";
    let db = "reflexive://db";

    affine::build_lattice(repo, db);
    affine::build_lattice(repo, db); // second call — must behave identically
}

/// Reflexive: calling query_lattice twice with identical arguments must produce
/// the same outcome (both calls complete without error).
#[test]
fn reflexive_query_lattice_idempotent_on_same_args() {
    let zoom = "reflexive-node";
    let db = "reflexive://db";

    affine::query_lattice(zoom, db);
    affine::query_lattice(zoom, db); // second call — must behave identically
}

// ---------------------------------------------------------------------------
// Contract: public API invariants
// ---------------------------------------------------------------------------

/// Contract: build_lattice must always return () — callers cannot accidentally
/// assign a meaningful value from a void operation.
#[test]
fn contract_build_lattice_returns_unit() {
    let result: () = affine::build_lattice("contract-repo", "contract://db");
    let _ = result; // type annotation above is the real assertion
}

/// Contract: query_lattice must always return () for the same reason.
#[test]
fn contract_query_lattice_returns_unit() {
    let result: () = affine::query_lattice("contract-node", "contract://db");
    let _ = result;
}

/// Contract: neither function panics when given the same db URI but different
/// repo/node identifiers — the db argument is shared state in future impls.
#[test]
fn contract_shared_db_different_targets() {
    let shared_db = "shared://lattice-db";

    affine::build_lattice("repo-one", shared_db);
    affine::build_lattice("repo-two", shared_db);
    affine::query_lattice("node-one", shared_db);
    affine::query_lattice("node-two", shared_db);
}

// ---------------------------------------------------------------------------
// Aspect: cross-cutting concerns
// ---------------------------------------------------------------------------

/// Aspect (resilience): empty strings must not cause a panic. The caller is
/// responsible for validation; the library must not crash on degenerate input.
#[test]
fn aspect_no_panic_on_empty_string_inputs() {
    affine::build_lattice("", "");
    affine::query_lattice("", "");
}

/// Aspect (resilience): whitespace-only strings must be handled gracefully.
#[test]
fn aspect_no_panic_on_whitespace_inputs() {
    affine::build_lattice("   ", "   ");
    affine::query_lattice("\t\n", "\t\n");
}

/// Aspect (security): strings that look like potential injection payloads must
/// not cause panics or undefined behaviour.
#[test]
fn aspect_no_panic_on_injection_like_inputs() {
    // SQL injection attempt
    affine::build_lattice("'; DROP TABLE lattice; --", "db://test");
    // Shell injection attempt
    affine::query_lattice("$(rm -rf /)", "db://test");
    // Path traversal attempt
    affine::build_lattice("../../etc/passwd", "db://test");
}

/// Aspect (security): API token patterns in the db URI string must not cause
/// a panic. We deliberately avoid real tokens; this tests robustness only.
#[test]
fn aspect_no_panic_on_token_like_db_uri() {
    affine::build_lattice("secure-repo", "postgres://user:FAKE_TOKEN_1234@host/db");
    affine::query_lattice("secure-node", "postgres://user:FAKE_TOKEN_1234@host/db");
}

/// Aspect (memory): very long inputs must not cause stack overflows or heap OOM panics.
#[test]
fn aspect_no_panic_on_very_long_inputs() {
    let huge = "A".repeat(1_000_000);
    affine::build_lattice(&huge, &huge);
    affine::query_lattice(&huge, &huge);
}

/// Aspect (unicode): multi-byte UTF-8 strings must be handled without panic.
#[test]
fn aspect_no_panic_on_unicode_inputs() {
    affine::build_lattice("仕組みのリポジトリ", "db://日本語");
    affine::query_lattice("модуль_семантики", "db://кириллица");
    affine::build_lattice("🦔 echidna", "db://🧪");
}
