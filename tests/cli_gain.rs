mod common;

use common::{assert_success, read, remove_repo, run_init, temp_repo, write};
use std::path::Path;
use std::process::Command;

fn run_gain(repo_root: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_fgl"))
        .current_dir(repo_root)
        .arg("gain")
        .output()
        .expect("run fgl gain")
}

fn run_pack(repo_root: &Path, active: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_fgl"))
        .current_dir(repo_root)
        .arg("pack")
        .arg(active)
        .output()
        .expect("run fgl pack")
}

#[test]
fn gain_prints_zero_summary_without_history() {
    let repo = temp_repo();

    let output = run_gain(&repo);

    assert_success(&output);
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout utf8"),
        "packs=0 raw=0 pack=0 saved=0 savings=0.00 prefix=0 active=0\n"
    );

    remove_repo(&repo);
}

#[test]
fn gain_summarizes_pack_history_after_pack_runs() {
    let repo = temp_repo();

    assert_success(&run_init(&repo));
    write(&repo, "src/module.py", "def wave():\n    return 1\n");
    write(&repo, "docs/active.md", "# focus\n");

    assert_success(&run_pack(&repo, "docs/active.md"));
    write(&repo, "docs/active.md", "# focus\n\nnext\n");
    assert_success(&run_pack(&repo, "docs/active.md"));

    let output = run_gain(&repo);

    assert_success(&output);

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(stdout.contains("packs=2 "), "stdout was {stdout:?}");
    assert!(stdout.contains("raw="), "stdout was {stdout:?}");
    assert!(stdout.contains("pack="), "stdout was {stdout:?}");
    assert!(stdout.contains("saved="), "stdout was {stdout:?}");
    assert!(stdout.contains("prefix="), "stdout was {stdout:?}");
    assert!(stdout.contains("active="), "stdout was {stdout:?}");

    let history = read(&repo, ".fgl/history.jsonl");
    assert_eq!(history.lines().count(), 2, "history was {history:?}");

    remove_repo(&repo);
}
