use std::fs;
use std::path::PathBuf;

use frugal::languages::skeletonize;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("languages")
        .join("go")
}

fn assert_fixture(name: &str) {
    let source = fs::read_to_string(fixture_dir().join(format!("{name}.go")))
        .expect("read go fixture source");
    let expected = fs::read_to_string(fixture_dir().join(format!("{name}.skeleton")))
        .expect("read go fixture skeleton");
    let virtual_path = PathBuf::from(format!("pkg/{name}.go"));

    let output = skeletonize(&virtual_path, &source);

    assert_eq!(output.fence_label, "go");
    assert!(!output.is_placeholder, "fixture {name} marked placeholder");
    assert_eq!(
        output.body, expected,
        "skeleton mismatch for fixture {name}"
    );
}

#[test]
fn top_level_fixture_matches_expected_skeleton() {
    assert_fixture("top_level");
}

#[test]
fn grouped_fixture_matches_expected_skeleton() {
    assert_fixture("grouped");
}
