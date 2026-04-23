use std::fs;
use std::path::PathBuf;

use frugal::languages::skeletonize;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("languages")
        .join("css")
}

fn assert_fixture(name: &str) {
    let source = fs::read_to_string(fixture_dir().join(format!("{name}.css")))
        .expect("read css fixture source");
    let expected = fs::read_to_string(fixture_dir().join(format!("{name}.skeleton")))
        .expect("read css fixture skeleton");
    let virtual_path = PathBuf::from(format!("web/{name}.css"));

    let output = skeletonize(&virtual_path, &source);

    assert_eq!(output.fence_label, "css");
    assert!(!output.is_placeholder, "fixture {name} marked placeholder");
    assert_eq!(
        output.body, expected,
        "skeleton mismatch for fixture {name}"
    );
}

#[test]
fn selectors_fixture_matches_expected_skeleton() {
    assert_fixture("selectors");
}

#[test]
fn media_fixture_matches_expected_skeleton() {
    assert_fixture("media");
}

#[test]
fn output_is_deterministic_across_parses() {
    let path = PathBuf::from("web/app.css");
    let source = ".app { display: grid; }\n@media (min-width: 40rem) { .app { gap: 1rem; } }\n";

    let first = skeletonize(&path, source);
    let second = skeletonize(&path, source);

    assert_eq!(first.body, second.body);
    assert!(!first.is_placeholder);
    assert!(!first.body.contains("TODO"));
}
