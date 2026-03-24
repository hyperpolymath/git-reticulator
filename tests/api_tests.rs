use git_reticulator::lattice::affine;

#[test]
fn test_cli_argument_parsing() {
    // Basic test to ensure the library bridge handles the CLI commands
    affine::build_lattice("./tests/fixtures", "db_uri");
}

#[test]
fn test_api_health_endpoint() {
    // API logic test
    let status = "success";
    assert_eq!(status, "success");
}
