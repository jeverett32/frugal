mod common;

use common::{assert_success, remove_repo, temp_repo};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn fixture_root(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("status")
        .join(name)
}

fn copy_fixture_dir(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).expect("create fixture target");

    for entry in fs::read_dir(src).expect("read fixture dir") {
        let entry = entry.expect("read fixture entry");
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if entry.file_type().expect("fixture type").is_dir() {
            copy_fixture_dir(&src_path, &dst_path);
        } else {
            fs::copy(&src_path, &dst_path).expect("copy fixture file");
        }
    }
}

fn repo_from_fixture(name: &str) -> PathBuf {
    let repo = temp_repo();
    copy_fixture_dir(&fixture_root(name), &repo);
    repo
}

fn run_status(repo_root: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_fgl"))
        .current_dir(repo_root)
        .arg("status")
        .args(args)
        .output()
        .expect("run fgl status")
}

#[test]
fn status_prints_exact_one_line_shape() {
    let repo = repo_from_fixture("shared_repo");

    let output = run_status(&repo, &["docs/active.md"]);

    assert_success(&output);
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout utf8"),
        "prefix=13 active=2 ratio=6.50 files=4 langs=2\n"
    );
    assert_eq!(String::from_utf8(output.stderr).expect("stderr utf8"), "");

    remove_repo(&repo);
}

#[test]
fn status_uses_ceil_bytes_divided_by_four_token_estimate() {
    let repo = repo_from_fixture("shared_repo");

    let output = run_status(&repo, &["docs/active.md"]);

    assert_success(&output);

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(stdout.contains("prefix=13 "), "stdout was {stdout:?}");
    assert!(stdout.contains("active=2 "), "stdout was {stdout:?}");

    remove_repo(&repo);
}

#[test]
fn status_matches_pack_selection_pipeline() {
    let repo = repo_from_fixture("shared_repo");

    let status = run_status(&repo, &["docs/active.md"]);
    let pack = Command::new(env!("CARGO_BIN_EXE_fgl"))
        .current_dir(&repo)
        .arg("pack")
        .arg("docs/active.md")
        .output()
        .expect("run fgl pack");

    assert_success(&status);
    assert_success(&pack);

    let status_stdout = String::from_utf8(status.stdout).expect("status stdout utf8");
    assert!(
        status_stdout.contains("files=4 "),
        "stdout was {status_stdout:?}"
    );
    assert!(
        status_stdout.contains("langs=2"),
        "stdout was {status_stdout:?}"
    );

    let pack_stdout = String::from_utf8(pack.stdout).expect("pack stdout utf8");
    assert!(pack_stdout.contains("AGENTS.md"));
    assert!(pack_stdout.contains("CLAUDE.md"));
    assert!(pack_stdout.contains("src/module.py"));
    assert!(pack_stdout.contains("docs/active.md"));

    remove_repo(&repo);
}
