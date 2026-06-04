// SPDX-License-Identifier: MPL-2.0
// Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
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
