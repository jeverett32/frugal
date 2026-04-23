use std::fs;
use std::path::Path;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_repo() -> std::path::PathBuf {
    let unique = format!(
        "frugal-smoke-test-{}-{}",
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

fn write(repo_root: &Path, relative: &str, contents: &str) {
    let path = repo_root.join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }
    fs::write(path, contents).expect("write file");
}

fn run_fgl(repo_root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_fgl"))
        .current_dir(repo_root)
        .args(args)
        .output()
        .expect("run fgl")
}

#[test]
fn root_help_succeeds() {
    let repo = temp_repo();

    let output = run_fgl(&repo, &["--help"]);

    assert!(
        output.status.success(),
        "expected success\nstatus: {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Predictable prompt-pack assembly and repo bootstrap CLI"));
    assert!(stdout.contains("init"));
    assert!(stdout.contains("pack"));
    assert!(stdout.contains("status"));

    let _ = fs::remove_dir_all(repo);
}

#[test]
fn init_fails_on_malformed_config() {
    let repo = temp_repo();
    write(&repo, ".fgl/config.toml", "version = [\n");

    let output = run_fgl(&repo, &["init"]);

    assert!(!output.status.success(), "expected failure");
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("config error: invalid TOML:"));

    let _ = fs::remove_dir_all(repo);
}

#[test]
fn init_fails_on_malformed_managed_markers() {
    let repo = temp_repo();
    write(
        &repo,
        "AGENTS.md",
        "<!-- frugal:managed:start -->\nmissing end\n",
    );

    let output = run_fgl(&repo, &["init"]);

    assert!(!output.status.success(), "expected failure");
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("marker error: missing managed block end marker"));

    let _ = fs::remove_dir_all(repo);
}
