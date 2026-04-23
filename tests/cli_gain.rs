mod common;

use common::{assert_success, read, remove_repo, run_init, temp_repo, write};
use std::path::Path;
use std::process::Command;

fn run_gain(repo_root: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_fgl"))
        .current_dir(repo_root)
        .arg("gain")
        .args(args)
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

fn run_gain_with_home(repo_root: &Path, args: &[&str], home: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_fgl"))
        .current_dir(repo_root)
        .env("HOME", home)
        .arg("gain")
        .args(args)
        .output()
        .expect("run fgl gain")
}

#[test]
fn gain_prints_zero_summary_without_history() {
    let repo = temp_repo();

    let output = run_gain(&repo, &[]);

    assert_success(&output);
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(
        stdout.contains("FGL Estimated Savings (Repo Scope)"),
        "stdout was {stdout:?}"
    );
    assert!(
        stdout.contains("Total packs:      0"),
        "stdout was {stdout:?}"
    );
    assert!(
        stdout.contains("Raw tokens:       0"),
        "stdout was {stdout:?}"
    );
    assert!(
        stdout.contains("Pack tokens:      0"),
        "stdout was {stdout:?}"
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

    let output = run_gain(&repo, &[]);

    assert_success(&output);

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(
        stdout.contains("Total packs:      2"),
        "stdout was {stdout:?}"
    );
    assert!(stdout.contains("Top Active Files"), "stdout was {stdout:?}");
    assert!(stdout.contains("docs/active.md"), "stdout was {stdout:?}");

    let history = read(&repo, ".fgl/history.jsonl");
    assert_eq!(history.lines().count(), 2, "history was {history:?}");

    remove_repo(&repo);
}

#[test]
fn gain_json_emits_machine_readable_report() {
    let repo = temp_repo();

    assert_success(&run_init(&repo));
    write(&repo, "src/module.py", "def wave():\n    return 1\n");
    write(&repo, "docs/active.md", "# focus\n");
    assert_success(&run_pack(&repo, "docs/active.md"));

    let output = run_gain(&repo, &["--json"]);

    assert_success(&output);
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(stdout.contains("\"summary\""), "stdout was {stdout:?}");
    assert!(
        stdout.contains("\"top_active_files\""),
        "stdout was {stdout:?}"
    );
    assert!(stdout.contains("\"recent_runs\""), "stdout was {stdout:?}");

    remove_repo(&repo);
}

#[test]
fn gain_history_shows_recent_runs() {
    let repo = temp_repo();

    assert_success(&run_init(&repo));
    write(&repo, "src/module.py", "def wave():\n    return 1\n");
    write(&repo, "docs/active.md", "# focus\n");
    assert_success(&run_pack(&repo, "docs/active.md"));

    let output = run_gain(&repo, &["--history", "--limit", "5"]);

    assert_success(&output);
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(stdout.contains("Recent Runs"), "stdout was {stdout:?}");
    assert!(stdout.contains("ts="), "stdout was {stdout:?}");
    assert!(stdout.contains("docs/active.md"), "stdout was {stdout:?}");

    remove_repo(&repo);
}

#[test]
fn gain_summary_includes_savings_bar() {
    let repo = temp_repo();

    assert_success(&run_init(&repo));
    write(&repo, "src/module.py", "def wave():\n    return 1\n");
    write(&repo, "docs/active.md", "# focus\n");
    assert_success(&run_pack(&repo, "docs/active.md"));

    let output = run_gain(&repo, &[]);

    assert_success(&output);
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(
        stdout.contains('█') || stdout.contains('░'),
        "savings bar missing from output: {stdout:?}"
    );

    remove_repo(&repo);
}

#[test]
fn gain_global_shows_registered_repos() {
    let fake_home = temp_repo();
    let repo = temp_repo();

    assert_success(&run_init(&repo));
    write(&repo, "src/module.py", "def wave():\n    return 1\n");
    write(&repo, "docs/active.md", "# focus\n");

    // Pack registers the repo in the global registry via HOME env
    let pack_out = Command::new(env!("CARGO_BIN_EXE_fgl"))
        .current_dir(&repo)
        .env("HOME", &fake_home)
        .arg("pack")
        .arg("docs/active.md")
        .output()
        .expect("run fgl pack");
    assert_success(&pack_out);

    let output = run_gain_with_home(&repo, &["--global"], &fake_home);

    assert_success(&output);
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(
        stdout.contains("Global Scope"),
        "global header missing: {stdout:?}"
    );

    remove_repo(&fake_home);
    remove_repo(&repo);
}

#[test]
fn gain_global_no_repos_shows_helpful_message() {
    let fake_home = temp_repo();
    let repo = temp_repo();

    assert_success(&run_init(&repo));

    let output = run_gain_with_home(&repo, &["--global"], &fake_home);

    assert_success(&output);
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(
        stdout.contains("No repos registered"),
        "empty global message missing: {stdout:?}"
    );

    remove_repo(&fake_home);
    remove_repo(&repo);
}

#[test]
fn gain_global_json_emits_repos_and_grand_summary() {
    let fake_home = temp_repo();
    let repo = temp_repo();

    assert_success(&run_init(&repo));
    write(&repo, "src/module.py", "def wave():\n    return 1\n");
    write(&repo, "docs/active.md", "# focus\n");
    let pack_out = Command::new(env!("CARGO_BIN_EXE_fgl"))
        .current_dir(&repo)
        .env("HOME", &fake_home)
        .arg("pack")
        .arg("docs/active.md")
        .output()
        .expect("run fgl pack");
    assert_success(&pack_out);

    let output = run_gain_with_home(&repo, &["--global", "--json"], &fake_home);

    assert_success(&output);
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(
        stdout.contains("\"repos\""),
        "repos key missing: {stdout:?}"
    );
    assert!(
        stdout.contains("\"grand_summary\""),
        "grand_summary key missing: {stdout:?}"
    );

    remove_repo(&fake_home);
    remove_repo(&repo);
}
