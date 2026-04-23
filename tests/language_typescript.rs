use std::fs;
use std::path::PathBuf;

use frugal::languages::skeletonize;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("languages")
        .join("typescript")
}

fn assert_fixture(name: &str) {
    let source = fs::read_to_string(fixture_dir().join(format!("{name}.ts")))
        .expect("read typescript fixture source");
    let expected = fs::read_to_string(fixture_dir().join(format!("{name}.skeleton")))
        .expect("read typescript fixture skeleton");
    let virtual_path = PathBuf::from(format!("web/{name}.ts"));

    let output = skeletonize(&virtual_path, &source);

    assert_eq!(output.fence_label, "typescript");
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
fn tsx_path_uses_tsx_parser() {
    let path = PathBuf::from("web/component.tsx");
    let source = "export const Button = (props: { label: string }) => props.label;\n";
    let output = skeletonize(&path, source);
    assert_eq!(output.fence_label, "typescript");
    assert!(!output.is_placeholder);
    assert!(!output.body.contains("TODO"));
    assert!(output.body.contains("export const Button"));
}

#[test]
fn output_is_deterministic_across_parses() {
    let path = PathBuf::from("web/app.ts");
    let source = "export interface A { x: number; }\nexport type B = A;\n";
    let first = skeletonize(&path, source);
    let second = skeletonize(&path, source);
    assert_eq!(first.body, second.body);
    assert!(!first.is_placeholder);
}
