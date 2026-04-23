mod common;

use common::{assert_success, remove_repo, temp_repo};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn fixture_root(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("pack")
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

#[test]
fn pack_markdown_matches_shared_repo_fixture_shape() {
    let repo = repo_from_fixture("shared_repo");

    let output = Command::new(env!("CARGO_BIN_EXE_fgl"))
        .current_dir(&repo)
        .arg("pack")
        .arg("docs/active.md")
        .output()
        .expect("run fgl pack");

    assert_success(&output);

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(stdout.contains("# Foundation\n"), "stdout was {stdout:?}");
    assert!(stdout.contains("# Secondary Skeletons\n"), "stdout was {stdout:?}");
    assert!(stdout.contains("# Active Zone\n"), "stdout was {stdout:?}");
    assert!(stdout.contains("AGENTS.md"), "stdout was {stdout:?}");
    assert!(stdout.contains("CLAUDE.md"), "stdout was {stdout:?}");
    assert!(stdout.contains("src/module.py"), "stdout was {stdout:?}");
    assert!(stdout.contains("docs/active.md"), "stdout was {stdout:?}");

    let foundation = stdout.find("# Foundation\n").expect("foundation heading");
    let secondary = stdout
        .find("# Secondary Skeletons\n")
        .expect("secondary heading");
    let active = stdout.find("# Active Zone\n").expect("active heading");
    assert!(foundation < secondary, "stdout was {stdout:?}");
    assert!(secondary < active, "stdout was {stdout:?}");

    remove_repo(&repo);
}
