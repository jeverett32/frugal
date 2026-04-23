use crate::app::StatusCommand;
use crate::cli::StatusArgs;
use crate::discovery::{build_selection, SelectedPath, Selection};
use crate::error::{Error, Result};
use crate::languages;
use crate::token::estimate_tokens;
use std::collections::BTreeSet;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Clone, Copy)]
pub struct StatusRunner;

impl StatusCommand for StatusRunner {
    fn run(&self, args: &StatusArgs) -> Result<()> {
        let cwd = std::env::current_dir().map_err(Error::io)?;
        let selection = select_inputs(&cwd, &args.paths)?;
        let summary = StatusSummary::from_selection(&selection);

        let mut stdout = io::stdout().lock();
        writeln!(stdout, "{summary}").map_err(Error::io)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusSummary {
    pub prefix_tokens: usize,
    pub active_tokens: usize,
    pub files: usize,
    pub languages: usize,
}

impl StatusSummary {
    pub fn from_selection(selection: &[SelectedFile]) -> Self {
        let prefix_bytes = selection
            .iter()
            .filter(|file| file.role == FileRole::Prefix)
            .map(|file| file.bytes)
            .sum();
        let active_bytes = selection
            .iter()
            .filter(|file| file.role == FileRole::Active)
            .map(|file| file.bytes)
            .sum();
        let languages = selection
            .iter()
            .filter_map(|file| file.language.as_deref())
            .collect::<BTreeSet<_>>()
            .len();

        Self {
            prefix_tokens: estimate_tokens(prefix_bytes),
            active_tokens: estimate_tokens(active_bytes),
            files: selection.len(),
            languages,
        }
    }

    fn ratio_string(&self) -> String {
        if self.active_tokens == 0 {
            "inf".to_string()
        } else {
            format!(
                "{:.2}",
                self.prefix_tokens as f64 / self.active_tokens as f64
            )
        }
    }
}

impl std::fmt::Display for StatusSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "prefix={} active={} ratio={} files={} langs={}",
            self.prefix_tokens,
            self.active_tokens,
            self.ratio_string(),
            self.files,
            self.languages
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectedFile {
    pub relative_path: PathBuf,
    pub role: FileRole,
    pub language: Option<String>,
    pub bytes: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileRole {
    Prefix,
    Active,
}

pub fn select_inputs(repo_root: &Path, active_paths: &[PathBuf]) -> Result<Vec<SelectedFile>> {
    let config = crate::init::load_config(repo_root)?.unwrap_or_default();
    let selection = build_selection(repo_root, &config, active_paths)?;

    selection_to_inputs(&selection)
}

fn selection_to_inputs(selection: &Selection) -> Result<Vec<SelectedFile>> {
    let mut selected = Vec::new();

    for path in &selection.foundation {
        selected.push(selected_file(path, FileRole::Prefix)?);
    }

    for path in &selection.secondary {
        selected.push(selected_file(path, FileRole::Prefix)?);
    }

    for path in &selection.active {
        selected.push(selected_file(path, FileRole::Active)?);
    }

    Ok(selected)
}

fn selected_file(path: &SelectedPath, role: FileRole) -> Result<SelectedFile> {
    let bytes = fs::read(&path.absolute_path).map_err(Error::io)?.len();
    let language = path
        .language
        .label()
        .or_else(|| languages::fence_label_for_path(&path.repo_relative_path))
        .map(str::to_string);

    Ok(SelectedFile {
        relative_path: path.repo_relative_path.clone(),
        role,
        language,
        bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::{select_inputs, FileRole, SelectedFile, StatusSummary};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_repo() -> PathBuf {
        let unique = format!(
            "frugal-status-test-{}-{}",
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

    fn write(repo_root: &std::path::Path, relative: &str, contents: &str) {
        let path = repo_root.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent dirs");
        }
        fs::write(path, contents).expect("write file");
    }

    #[test]
    fn summary_formats_exact_one_line_shape() {
        let summary = StatusSummary {
            prefix_tokens: 17,
            active_tokens: 6,
            files: 4,
            languages: 2,
        };

        assert_eq!(
            summary.to_string(),
            "prefix=17 active=6 ratio=2.83 files=4 langs=2"
        );
    }

    #[test]
    fn summary_uses_inf_ratio_when_active_zero() {
        let summary = StatusSummary {
            prefix_tokens: 3,
            active_tokens: 0,
            files: 2,
            languages: 1,
        };

        assert_eq!(
            summary.to_string(),
            "prefix=3 active=0 ratio=inf files=2 langs=1"
        );
    }

    #[test]
    fn selection_uses_shared_discovery_seam() {
        let repo = temp_repo();
        write(
            &repo,
            ".fgl/config.toml",
            "version = 1\n\n[foundation]\npinned = [\"AGENTS.md\", \"CLAUDE.md\"]\n\n[languages]\nenabled = [\"python\", \"rust\", \"javascript\", \"typescript\", \"go\"]\n",
        );
        write(&repo, "AGENTS.md", "agent rules");
        write(&repo, "CLAUDE.md", "claude guide");
        write(&repo, "src/module.py", "def wave():\n    return 1\n");
        write(&repo, "docs/active.md", "abcde");

        let selected = select_inputs(&repo, &[PathBuf::from("docs/active.md")]).expect("select");

        assert_eq!(
            selected,
            vec![
                SelectedFile {
                    relative_path: PathBuf::from("AGENTS.md"),
                    role: FileRole::Prefix,
                    language: Some("markdown".to_string()),
                    bytes: 11,
                },
                SelectedFile {
                    relative_path: PathBuf::from("CLAUDE.md"),
                    role: FileRole::Prefix,
                    language: Some("markdown".to_string()),
                    bytes: 12,
                },
                SelectedFile {
                    relative_path: PathBuf::from("src/module.py"),
                    role: FileRole::Prefix,
                    language: Some("python".to_string()),
                    bytes: 25,
                },
                SelectedFile {
                    relative_path: PathBuf::from("docs/active.md"),
                    role: FileRole::Active,
                    language: Some("markdown".to_string()),
                    bytes: 5,
                },
            ]
        );

        let _ = fs::remove_dir_all(repo);
    }
}
