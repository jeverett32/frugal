use std::fs;
use std::path::PathBuf;

use frugal::languages::skeletonize;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("languages")
        .join("markdown")
}

#[test]
fn basic_fixture_matches_expected_skeleton() {
    let source = fs::read_to_string(fixture_dir().join("basic.md")).expect("read markdown");
    let expected =
        fs::read_to_string(fixture_dir().join("basic.skeleton")).expect("read markdown skeleton");
    let output = skeletonize(&PathBuf::from("docs/basic.md"), &source);

    assert_eq!(output.fence_label, "markdown");
    assert!(!output.is_placeholder);
    assert_eq!(output.body, expected);
}
