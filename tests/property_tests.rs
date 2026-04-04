// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// tests/property_tests.rs
// Property-based (P2P) tests for git-reticulator using proptest.
//
// Properties verified:
//   P1 - Any non-empty string is accepted as a repo path without panicking.
//   P2 - Any non-empty string is accepted as a db URI without panicking.
//   P3 - build_lattice never panics on arbitrary valid UTF-8 string pairs.
//   P4 - query_lattice never panics on arbitrary valid UTF-8 string pairs.
//   P5 - Calling both functions in sequence never panics.

use git_reticulator::lattice::affine;
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// P1 + P2 + P3: build_lattice accepts arbitrary non-empty string pairs
// ---------------------------------------------------------------------------
proptest! {
    /// Any non-empty repo path and db URI must not cause build_lattice to panic.
    #[test]
    fn prop_build_lattice_no_panic_non_empty(
        repo in ".+",   // one or more arbitrary chars
        db   in ".+",
    ) {
        affine::build_lattice(&repo, &db);
    }
}

// ---------------------------------------------------------------------------
// P4: query_lattice accepts arbitrary non-empty strings
// ---------------------------------------------------------------------------
proptest! {
    /// Any non-empty zoom node and db URI must not cause query_lattice to panic.
    #[test]
    fn prop_query_lattice_no_panic_non_empty(
        zoom in ".+",
        db   in ".+",
    ) {
        affine::query_lattice(&zoom, &db);
    }
}

// ---------------------------------------------------------------------------
// P5: Chained build + query never panics
// ---------------------------------------------------------------------------
proptest! {
    /// build_lattice followed immediately by query_lattice on the same db must
    /// not panic regardless of input values.
    #[test]
    fn prop_build_then_query_no_panic(
        repo in ".+",
        node in ".+",
        db   in ".+",
    ) {
        affine::build_lattice(&repo, &db);
        affine::query_lattice(&node, &db);
    }
}

// ---------------------------------------------------------------------------
// P6: Empty strings (edge-case) also do not panic
// ---------------------------------------------------------------------------
proptest! {
    /// build_lattice must not panic even when both arguments are the empty string.
    #[test]
    fn prop_build_lattice_empty_strings(_seed in 0u8..=255) {
        // Parameterised only to satisfy proptest's requirement of at least one
        // variable; the actual inputs under test are always empty strings.
        affine::build_lattice("", "");
    }
}

proptest! {
    /// query_lattice must not panic even when both arguments are the empty string.
    #[test]
    fn prop_query_lattice_empty_strings(_seed in 0u8..=255) {
        affine::query_lattice("", "");
    }
}
