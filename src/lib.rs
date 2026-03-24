#![forbid(unsafe_code)]
// src/lib.rs
// Rust bridge for Git-Reticulator, connecting the Rust CLI/API
// to the core AffineScript lattice logic.

pub mod api {
    pub mod app;
}

pub mod lattice {
    pub mod affine {
        // This is a placeholder for the actual AffineScript integration.
        // In a real environment, this might use the `affinescript` crate
        // to load and execute `.as` files.

        pub fn build_lattice(repo: &str, db: &str) {
            println!("Reticulating repo: {} into DB: {}", repo, db);
            // Logic to call src/lattice/affine/lattice.as via affinescript engine
        }

        pub fn query_lattice(zoom: &str, db: &str) {
            println!("Zooming into semantic node: {} from DB: {}", zoom, db);
            // Logic to call zoom_to_node in src/lattice/affine/lattice.as
        }
    }
}
