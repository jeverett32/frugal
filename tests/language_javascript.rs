use std::fs;
use std::path::PathBuf;

use frugal::languages::skeletonize;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("languages")
        .join("javascript")
}

fn assert_fixture(name: &str) {
    let source = fs::read_to_string(fixture_dir().join(format!("{name}.js")))
        .expect("read javascript fixture source");
    let expected = fs::read_to_string(fixture_dir().join(format!("{name}.skeleton")))
        .expect("read javascript fixture skeleton");
    let virtual_path = PathBuf::from(format!("web/{name}.js"));

    let output = skeletonize(&virtual_path, &source);

    assert_eq!(output.fence_label, "javascript");
    assert!(!output.is_placeholder, "fixture {name} marked placeholder");
    assert_eq!(
        output.body, expected,
        "skeleton mismatch for fixture {name}"
    );
}

#[test]
fn basic_fixture_matches_expected_skeleton() {
    assert_fixture("basic");
}

#[test]
fn output_is_deterministic_across_parses() {
    let path = PathBuf::from("web/app.js");
    let source = "const A = 1;\nfunction f(x) { return x; }\nclass C { m() {} }\n";
    let first = skeletonize(&path, source);
    let second = skeletonize(&path, source);
    assert_eq!(first.body, second.body);
    assert!(!first.is_placeholder);
    assert!(!first.body.contains("TODO"));
}
