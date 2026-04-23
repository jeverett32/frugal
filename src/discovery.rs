use crate::config::Config;
use crate::error::{Error, PathOrigin, Result};
use crate::languages::{self, Language};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Selection contract for v0.1:
/// - foundation preserves config order
/// - active preserves CLI arg order
/// - secondary is derived last and must exclude any foundation/active path
/// - secondary includes only allowlisted source languages
/// - path-level dedupe remains as a defensive guard, not the primary precedence rule
/// - Linux/case-sensitive filesystem semantics are assumed for exact byte-lex ordering

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    pub foundation: Vec<SelectedPath>,
    pub secondary: Vec<SelectedPath>,
    pub active: Vec<SelectedPath>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectedPath {
    pub absolute_path: PathBuf,
    pub repo_relative_path: PathBuf,
    pub language: Language,
}

pub fn build_selection(
    repo_root: &Path,
    config: &Config,
    active_paths: &[PathBuf],
) -> Result<Selection> {
    let repo_root = fs::canonicalize(repo_root).map_err(Error::io)?;
    let active = active_paths
        .iter()
        .map(|path| to_selected_path(&repo_root, path, PathOrigin::Active))
        .collect::<Result<Vec<_>>>()?;
    let active_paths = active
        .iter()
        .map(|path| repo_path_key(&path.repo_relative_path))
        .collect::<HashSet<_>>();
    let foundation = config
        .foundation
        .pinned_paths
        .iter()
        .map(|path| to_selected_path(&repo_root, Path::new(path), PathOrigin::Foundation))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .filter(|path| !active_paths.contains(&repo_path_key(&path.repo_relative_path)))
        .collect::<Vec<_>>();
    let protected_paths = active
        .iter()
        .chain(foundation.iter())
        .map(|path| repo_path_key(&path.repo_relative_path))
        .collect::<HashSet<_>>();
    let secondary = discover_secondary(&repo_root, &protected_paths)?;

    Ok(dedup_selection(Selection {
        foundation,
        secondary,
        active,
    }))
}

fn discover_secondary(repo_root: &Path, protected_paths: &HashSet<String>) -> Result<Vec<SelectedPath>> {
    let mut relative_paths = Vec::new();
    collect_repo_files(repo_root, repo_root, protected_paths, &mut relative_paths)?;
    relative_paths.sort_by(|left, right| compare_repo_paths(left, right));

    relative_paths
        .into_iter()
        .map(|relative_path| to_secondary_selected_path(repo_root, relative_path))
        .collect()
}

fn collect_repo_files(
    root: &Path,
    current: &Path,
    protected_paths: &HashSet<String>,
    files: &mut Vec<PathBuf>,
) -> Result<()> {
    for entry in fs::read_dir(current).map_err(Error::io)? {
        let entry = entry.map_err(Error::io)?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(Error::io)?;
        let file_name = entry.file_name();

        if file_type.is_symlink() {
            continue;
        }

        if file_type.is_dir() {
            if skip_directory(&file_name) {
                continue;
            }

            collect_repo_files(root, &path, protected_paths, files)?;
            continue;
        }

        if file_type.is_file() {
            let relative_path = path
                .strip_prefix(root)
                .expect("walked path must remain under repo root")
                .to_path_buf();
            let path_key = repo_path_key(&relative_path);

            if protected_paths.contains(&path_key) || !languages::is_secondary_eligible(&relative_path) {
                continue;
            }

            files.push(relative_path);
        }
    }

    Ok(())
}

fn dedup_selection(selection: Selection) -> Selection {
    let mut seen = HashSet::new();

    Selection {
        foundation: dedup_slab(selection.foundation, &mut seen),
        secondary: dedup_slab(selection.secondary, &mut seen),
        active: dedup_slab(selection.active, &mut seen),
    }
}

fn dedup_slab(paths: Vec<SelectedPath>, seen: &mut HashSet<String>) -> Vec<SelectedPath> {
    paths.into_iter()
        .filter(|path| seen.insert(repo_path_key(&path.repo_relative_path)))
        .collect()
}

fn to_selected_path(repo_root: &Path, path: &Path, origin: PathOrigin) -> Result<SelectedPath> {
    let joined_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    };

    let absolute_path = match fs::canonicalize(&joined_path) {
        Ok(path) => path,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(Error::path_not_found(path.to_path_buf(), origin));
        }
        Err(error) => return Err(Error::io(error)),
    };

    let repo_relative_path = absolute_path
        .strip_prefix(repo_root)
        .map_err(|_| Error::path_outside_repo(path.to_path_buf(), origin))?
        .to_path_buf();

    Ok(SelectedPath {
        absolute_path,
        language: languages::language_for_path(&repo_relative_path),
        repo_relative_path,
    })
}

fn to_secondary_selected_path(repo_root: &Path, repo_relative_path: PathBuf) -> Result<SelectedPath> {
    let absolute_path = repo_root.join(&repo_relative_path);

    Ok(SelectedPath {
        absolute_path,
        language: languages::language_for_path(&repo_relative_path),
        repo_relative_path,
    })
}

fn compare_repo_paths(left: &Path, right: &Path) -> std::cmp::Ordering {
    repo_path_key(left).cmp(&repo_path_key(right))
}

fn repo_path_key(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn skip_directory(file_name: &std::ffi::OsStr) -> bool {
    matches!(
        file_name.to_string_lossy().as_ref(),
        ".git" | ".fgl" | "target" | "node_modules" | ".venv" | "__pycache__" | "dist" | "build"
    )
}

#[cfg(test)]
mod tests {
    use super::{build_selection, SelectedPath};
    use crate::config::Config;
    use crate::error::Error;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    #[test]
    fn foundation_preserves_config_order() {
        let repo = temp_repo("foundation_preserves_config_order");
        write_file(&repo, "z-last.md", "z");
        write_file(&repo, "a-first.md", "a");

        let mut config = Config::default();
        config.foundation.pinned_paths = vec!["z-last.md".into(), "a-first.md".into()];

        let selection = build_selection(&repo, &config, &[]).expect("selection builds");

        assert_paths(&selection.foundation, &["z-last.md", "a-first.md"]);
    }

    #[test]
    fn secondary_uses_byte_lexicographic_repo_relative_sort() {
        let repo = temp_repo("secondary_uses_byte_lexicographic_repo_relative_sort");
        write_file(&repo, "b.rs", "fn b() {}\n");
        write_file(&repo, "A.py", "def a():\n    return 1\n");
        write_file(&repo, "a.py", "def a():\n    return 2\n");
        write_file(&repo, "dir/B.go", "package dir\n");
        write_file(&repo, "dir/a.ts", "export const a = 1;\n");

        let selection = build_selection(&repo, &Config::default(), &[]).expect("selection builds");

        assert_paths(
            &selection.secondary,
            &["A.py", "a.py", "b.rs", "dir/B.go", "dir/a.ts"],
        );
    }

    #[test]
    fn active_preserves_cli_arg_order() {
        let repo = temp_repo("active_preserves_cli_arg_order");
        write_file(&repo, "first.md", "1");
        write_file(&repo, "second.md", "2");

        let selection = build_selection(
            &repo,
            &Config::default(),
            &[PathBuf::from("second.md"), PathBuf::from("first.md")],
        )
        .expect("selection builds");

        assert_paths(&selection.active, &["second.md", "first.md"]);
    }

    #[test]
    fn active_paths_are_excluded_from_secondary_and_preserved_in_active() {
        let repo = temp_repo("active_paths_are_excluded_from_secondary_and_preserved_in_active");
        write_file(&repo, "foundation.md", "f");
        write_file(&repo, "shared.md", "s");
        write_file(&repo, "active.md", "a");

        let mut config = Config::default();
        config.foundation.pinned_paths = vec!["foundation.md".into(), "shared.md".into()];

        let selection = build_selection(
            &repo,
            &config,
            &[PathBuf::from("shared.md"), PathBuf::from("active.md")],
        )
        .expect("selection builds");

        assert_paths(&selection.foundation, &["foundation.md"]);
        assert_paths(&selection.secondary, &[]);
        assert_paths(&selection.active, &["shared.md", "active.md"]);
    }

    #[test]
    fn secondary_excludes_fgl_junk_dirs_and_unsupported_extensions() {
        let repo = temp_repo("secondary_excludes_fgl_junk_dirs_and_unsupported_extensions");
        write_file(&repo, ".fgl/ignored.py", "print('ignore')");
        write_file(&repo, "target/ignored.rs", "fn ignore() {}");
        write_file(&repo, "node_modules/pkg/index.js", "export const x = 1;");
        write_file(&repo, "dist/bundle.ts", "export const y = 2;");
        write_file(&repo, "notes.md", "# note");
        write_file(&repo, "src/kept.py", "def kept():\n    return 1\n");
        write_file(&repo, "src/also_kept.rs", "fn kept() {}\n");

        let selection = build_selection(&repo, &Config::default(), &[]).expect("selection builds");

        assert_paths(&selection.secondary, &["src/also_kept.rs", "src/kept.py"]);
    }

    #[test]
    fn missing_active_path_reports_meaningful_error() {
        let repo = temp_repo("missing_active_path_reports_meaningful_error");

        let error = build_selection(&repo, &Config::default(), &[PathBuf::from("missing.md")])
            .expect_err("missing active path should error");

        assert_eq!(
            error,
            Error::path_not_found(PathBuf::from("missing.md"), crate::error::PathOrigin::Active)
        );
    }

    #[test]
    fn active_path_outside_repo_reports_meaningful_error() {
        let repo = temp_repo("active_path_outside_repo_reports_meaningful_error");
        let outside = std::env::temp_dir().join("frugal-outside.txt");
        fs::write(&outside, "outside").expect("outside file written");

        let error = build_selection(&repo, &Config::default(), &[outside.clone()])
            .expect_err("outside path should error");

        assert_eq!(
            error,
            Error::path_outside_repo(outside, crate::error::PathOrigin::Active)
        );
    }

    fn assert_paths(paths: &[SelectedPath], expected: &[&str]) {
        let actual = paths
            .iter()
            .map(|path| path.repo_relative_path.to_string_lossy().into_owned())
            .collect::<Vec<_>>();

        assert_eq!(actual, expected);
    }

    fn write_file(repo: &Path, relative_path: &str, contents: &str) {
        let path = repo.join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("parent dir created");
        }
        fs::write(path, contents).expect("file written");
    }

    fn temp_repo(label: &str) -> PathBuf {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);

        let path = std::env::temp_dir().join(format!(
            "frugal-discovery-{label}-{}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));

        if path.exists() {
            fs::remove_dir_all(&path).expect("old temp repo removed");
        }

        fs::create_dir_all(&path).expect("temp repo created");
        path
    }
}
