#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn temp_repo() -> PathBuf {
    let unique = format!(
        "frugal-init-test-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock before unix epoch")
            .as_nanos()
    );
    let path = std::env::temp_dir().join(unique);
    fs::create_dir_all(&path).expect("create temp repo");
    path
}

pub fn run_init(repo_root: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_fgl"))
        .current_dir(repo_root)
        .arg("init")
        .output()
        .expect("run fgl init")
}

pub fn run_init_rescan(repo_root: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_fgl"))
        .current_dir(repo_root)
        .args(["init", "--rescan"])
        .output()
        .expect("run fgl init --rescan")
}

pub fn read(repo_root: &Path, relative: &str) -> String {
    fs::read_to_string(repo_root.join(relative)).expect("read file")
}

pub fn write(repo_root: &Path, relative: &str, contents: &str) {
    let path = repo_root.join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }
    fs::write(path, contents).expect("write file");
}

pub fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "expected success\nstatus: {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

pub fn remove_repo(repo_root: &Path) {
    let _ = fs::remove_dir_all(repo_root);
}
