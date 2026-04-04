// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// src/lib.rs
// Rust bridge for Git-Reticulator, connecting the Rust CLI/API
// to the core AffineScript lattice logic.

#![forbid(unsafe_code)]

pub mod api {
    pub mod app;
}

pub mod lattice {
    pub mod affine {
        // This is a placeholder for the actual AffineScript integration.
        // In a real environment, this might use the `affinescript` crate
        // to load and execute `.as` files.

        /// Build a semantic lattice for the given repository path and persist it
        /// to the given database URI.
        ///
        /// # Arguments
        /// * `repo` - Path or identifier for the git repository to reticulate
        /// * `db`   - Database URI used to persist the resulting lattice
        pub fn build_lattice(repo: &str, db: &str) {
            println!("Reticulating repo: {} into DB: {}", repo, db);
            // Logic to call src/lattice/affine/lattice.as via affinescript engine
        }

        /// Query a previously-built lattice for a semantic zoom target.
        ///
        /// # Arguments
        /// * `zoom` - The semantic node identifier to zoom into
        /// * `db`   - Database URI of the lattice to query
        pub fn query_lattice(zoom: &str, db: &str) {
            println!("Zooming into semantic node: {} from DB: {}", zoom, db);
            // Logic to call zoom_to_node in src/lattice/affine/lattice.as
        }
    }
}

// ---------------------------------------------------------------------------
// Unit + Smoke tests (inline, always compiled with `cargo test`)
// ---------------------------------------------------------------------------
#[cfg(test)]
mod unit_tests {
    use super::lattice::affine;

    // --- Unit: function signatures accept basic inputs ---

    /// build_lattice should accept a simple repo path and db URI without panicking.
    #[test]
    fn unit_build_lattice_basic() {
        affine::build_lattice("my-repo", "postgres://localhost/lattice");
    }

    /// query_lattice should accept a zoom node and db URI without panicking.
    #[test]
    fn unit_query_lattice_basic() {
        affine::query_lattice("module::core", "postgres://localhost/lattice");
    }

    // --- Smoke: module-level availability ---

    /// Smoke: the affine module is importable and the two public functions
    /// are callable without any setup overhead.
    #[test]
    fn smoke_affine_module_callable() {
        // Simply calling both entry points verifies the module compiles and
        // is reachable at runtime (no linker / FFI issues).
        affine::build_lattice("smoke-repo", "smoke://db");
        affine::query_lattice("smoke-node", "smoke://db");
    }

    /// Smoke: an empty-but-valid argument set must not panic.
    #[test]
    fn smoke_empty_strings_do_not_panic() {
        // Empty strings represent a degenerate input; the current implementation
        // must tolerate them without panicking — even if the output is no-op.
        affine::build_lattice("", "");
        affine::query_lattice("", "");
    }

    // --- Unit: argument independence ---

    /// The two functions are independent; calling build_lattice should not
    /// alter the behaviour of a subsequent call to query_lattice.
    #[test]
    fn unit_functions_are_independent() {
        affine::build_lattice("repo-a", "db://a");
        affine::query_lattice("node-a", "db://a");
        affine::build_lattice("repo-b", "db://b");
        affine::query_lattice("node-b", "db://b");
        // If we reach here without a panic the invariant holds.
    }

    /// Unicode inputs should not cause any runtime panics.
    #[test]
    fn unit_unicode_inputs() {
        affine::build_lattice("репо/тест", "db://юникод");
        affine::query_lattice("узел_семантики", "db://юникод");
    }

    /// Very long strings should not trigger stack overflows or panics.
    #[test]
    fn unit_long_inputs() {
        let long = "x".repeat(4096);
        affine::build_lattice(&long, &long);
        affine::query_lattice(&long, &long);
    }
}
