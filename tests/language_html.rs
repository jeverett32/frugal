use std::fs;
use std::path::PathBuf;

use frugal::languages::skeletonize;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("languages")
        .join("html")
}

fn assert_fixture(name: &str) {
    let source = fs::read_to_string(fixture_dir().join(format!("{name}.html")))
        .expect("read html fixture source");
    let expected = fs::read_to_string(fixture_dir().join(format!("{name}.skeleton")))
        .expect("read html fixture skeleton");
    let virtual_path = PathBuf::from(format!("web/{name}.html"));

    let output = skeletonize(&virtual_path, &source);

    assert_eq!(output.fence_label, "html");
    assert!(!output.is_placeholder, "fixture {name} marked placeholder");
    assert_eq!(
        output.body, expected,
        "skeleton mismatch for fixture {name}"
    );
}

#[test]
fn semantic_fixture_matches_expected_skeleton() {
    assert_fixture("semantic");
}

#[test]
fn component_fixture_matches_expected_skeleton() {
    assert_fixture("components");
}

#[test]
fn output_is_deterministic_across_parses() {
    let path = PathBuf::from("web/app.html");
    let source = "<main id=\"app\"><h1>Dashboard</h1><section class=\"hero\"></section></main>";

    let first = skeletonize(&path, source);
    let second = skeletonize(&path, source);

    assert_eq!(first.body, second.body);
    assert!(!first.is_placeholder);
    assert!(!first.body.contains("TODO"));
}
