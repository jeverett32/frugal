use std::fs;
use std::path::PathBuf;

use frugal::languages::skeletonize;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("languages")
        .join("rust")
}

fn assert_fixture(name: &str) {
    let source = fs::read_to_string(fixture_dir().join(format!("{name}.rs")))
        .expect("read rust fixture source");
    let expected = fs::read_to_string(fixture_dir().join(format!("{name}.skeleton")))
        .expect("read rust fixture skeleton");
    let virtual_path = PathBuf::from(format!("pkg/{name}.rs"));

    let output = skeletonize(&virtual_path, &source);

    assert_eq!(output.fence_label, "rust");
    assert!(!output.is_placeholder, "fixture {name} marked placeholder");
    assert!(
        !output.body.contains("placeholder"),
        "fixture {name} still contains placeholder text"
    );
    assert_eq!(
        output.body, expected,
        "skeleton mismatch for fixture {name}"
    );
}

#[test]
fn api_surface_fixture_matches_expected_skeleton() {
    assert_fixture("api_surface");
}

#[test]
fn unsupported_only_fixture_collapses_to_ellipsis() {
    assert_fixture("unsupported_only");
}
