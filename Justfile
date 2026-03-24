# Justfile for Git-Reticulator orchestration

DB_URL := "postgresql://hyper:password@localhost/git_reticulator"

# Build all components
build:
    cargo build

# Build the semantic lattice of the current repository
reticulate repo_path=".":
    cargo run -- build --repo {{repo_path}} --db {{DB_URL}}

# Start the REST API server
serve:
    cargo run -- api --db {{DB_URL}}

# Run a sample query to zoom into the auth module
query-auth:
    cargo run -- query --zoom "auth" --db {{DB_URL}}

# Run tests (AffineScript and Rust)
test:
    cargo test
    # Add AffineScript-specific tests if available
    # as-test tests/lattice_tests.as

# Clean up build artifacts
clean:
    cargo clean

# Format code
fmt:
    cargo fmt --all

# Check formatting without modifying
fmt-check:
    cargo fmt --all --check

# Run panic-attacker pre-commit scan
assail:
    @command -v panic-attack >/dev/null 2>&1 && panic-attack assail . || echo "panic-attack not found — install from https://github.com/hyperpolymath/panic-attacker"
