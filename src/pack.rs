use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use crate::app::PackCommand;
use crate::cli::PackArgs;
use crate::discovery::{build_selection, SelectedPath, Selection};
use crate::error::{Error, Result};
use crate::languages;

#[derive(Debug, Default, Clone, Copy)]
pub struct PackRunner;

impl PackCommand for PackRunner {
    fn run(&self, args: &PackArgs) -> Result<()> {
        let cwd = std::env::current_dir().map_err(Error::io)?;
        let config = crate::init::load_config(&cwd)?.unwrap_or_default();
        let selection = build_selection(&cwd, &config, &args.paths)?;
        let pack = materialize_selection(&selection)?;
        let rendered = render_markdown(&pack);

        if let Some(output_path) = &args.output {
            fs::write(output_path, &rendered).map_err(Error::io)?;
        } else {
            let mut stdout = io::stdout().lock();
            stdout.write_all(rendered.as_bytes()).map_err(Error::io)?;
        }

        crate::gain::append_pack_history(&cwd, &selection, &rendered)
    }
}

/// Shared selection seam contract for markdown rendering.
///
/// `Selection` comes from discovery layer and must already satisfy:
/// - `foundation`: config-listed order
/// - `secondary`: repo-relative byte-lexicographic path order
/// - `active`: CLI arg order
/// - path-level dedupe already applied across all three slabs
///
/// Renderer loads raw file contents for foundation and active slabs. Secondary
/// slab routes through language registry, which may return deterministic
/// placeholders in wave 2.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PackSelection {
    pub foundation: Vec<PackFile>,
    pub secondary_skeletons: Vec<PackFile>,
    pub active_zone: Vec<PackFile>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackFile {
    pub path: PathBuf,
    pub fence_label: Option<&'static str>,
    pub body: String,
}

impl PackFile {
    pub fn new(
        path: impl Into<PathBuf>,
        fence_label: Option<&'static str>,
        body: impl Into<String>,
    ) -> Self {
        Self {
            path: path.into(),
            fence_label,
            body: body.into(),
        }
    }
}

pub fn materialize_selection(selection: &Selection) -> Result<PackSelection> {
    Ok(PackSelection {
        foundation: selection
            .foundation
            .iter()
            .map(load_raw_file)
            .collect::<Result<Vec<_>>>()?,
        secondary_skeletons: selection
            .secondary
            .iter()
            .map(load_secondary_skeleton)
            .collect::<Result<Vec<_>>>()?,
        active_zone: selection
            .active
            .iter()
            .map(load_raw_file)
            .collect::<Result<Vec<_>>>()?,
    })
}

pub fn render_markdown(selection: &PackSelection) -> String {
    let mut rendered = String::new();

    render_section("Foundation", &selection.foundation, &mut rendered);
    rendered.push('\n');
    render_section(
        "Secondary Skeletons",
        &selection.secondary_skeletons,
        &mut rendered,
    );
    rendered.push('\n');
    render_section("Active Zone", &selection.active_zone, &mut rendered);

    rendered
}

pub fn normalize_lf(input: &str) -> String {
    input.replace("\r\n", "\n").replace('\r', "\n")
}

fn load_raw_file(path: &SelectedPath) -> Result<PackFile> {
    let body = fs::read_to_string(&path.absolute_path).map_err(Error::io)?;

    Ok(PackFile::new(
        path.repo_relative_path.clone(),
        languages::fence_label_for_path(&path.repo_relative_path),
        body,
    ))
}

fn load_secondary_skeleton(path: &SelectedPath) -> Result<PackFile> {
    let source = fs::read_to_string(&path.absolute_path).map_err(Error::io)?;
    let skeleton = languages::skeletonize(&path.repo_relative_path, &source);

    Ok(PackFile::new(
        path.repo_relative_path.clone(),
        Some(skeleton.fence_label),
        skeleton.body,
    ))
}

fn render_section(title: &str, files: &[PackFile], rendered: &mut String) {
    rendered.push_str("# ");
    rendered.push_str(title);
    rendered.push('\n');
    rendered.push('\n');

    for (index, file) in files.iter().enumerate() {
        if index > 0 {
            rendered.push('\n');
        }

        let normalized_body = normalize_lf(&file.body);
        let fence = fence_for_body(&normalized_body);

        rendered.push_str("## `");
        rendered.push_str(&file.path.display().to_string());
        rendered.push_str("`\n");
        rendered.push('\n');
        rendered.push_str(&fence);

        if let Some(label) = file.fence_label {
            rendered.push_str(label);
        }

        rendered.push('\n');
        rendered.push_str(&normalized_body);

        if !normalized_body.ends_with('\n') {
            rendered.push('\n');
        }

        rendered.push_str(&fence);
        rendered.push('\n');
    }
}

fn fence_for_body(body: &str) -> String {
    let longest = longest_backtick_run(body);
    let width = longest.saturating_add(1).max(3);
    "`".repeat(width)
}

fn longest_backtick_run(body: &str) -> usize {
    let mut longest = 0;
    let mut current = 0;

    for ch in body.chars() {
        if ch == '`' {
            current += 1;
            longest = longest.max(current);
        } else {
            current = 0;
        }
    }

    longest
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::discovery::build_selection;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    #[test]
    fn renders_stable_markdown_contract_in_fixed_section_order() {
        let selection = PackSelection {
            foundation: vec![PackFile::new("AGENTS.md", Some("markdown"), "base\n")],
            secondary_skeletons: vec![PackFile::new(
                "src/lib.rs",
                Some("text"),
                "TODO: rust skeleton placeholder\npath: src/lib.rs\nstatus: deterministic placeholder; extraction not implemented\n",
            )],
            active_zone: vec![PackFile::new("notes.py", Some("python"), "print('focus')\n")],
        };

        assert_eq!(
            render_markdown(&selection),
            concat!(
                "# Foundation\n",
                "\n",
                "## `AGENTS.md`\n",
                "\n",
                "```markdown\n",
                "base\n",
                "```\n",
                "\n",
                "# Secondary Skeletons\n",
                "\n",
                "## `src/lib.rs`\n",
                "\n",
                "```text\n",
                "TODO: rust skeleton placeholder\n",
                "path: src/lib.rs\n",
                "status: deterministic placeholder; extraction not implemented\n",
                "```\n",
                "\n",
                "# Active Zone\n",
                "\n",
                "## `notes.py`\n",
                "\n",
                "```python\n",
                "print('focus')\n",
                "```\n",
            )
        );
    }

    #[test]
    fn materialize_selection_consumes_discovery_seam() {
        let repo = temp_repo("materialize_selection_consumes_discovery_seam");
        write_file(&repo, "AGENTS.md", "base\r\n");
        write_file(&repo, "src/lib.rs", "fn not_used() {}\n");
        write_file(&repo, "focus.py", "print('focus')\r\n");

        let mut config = Config::default();
        config.foundation.pinned = vec!["AGENTS.md".into()];

        let selection = build_selection(&repo, &config, &[PathBuf::from("focus.py")])
            .expect("selection builds");
        let pack = materialize_selection(&selection).expect("pack selection loads");

        assert_eq!(pack.foundation.len(), 1);
        assert_eq!(pack.secondary_skeletons.len(), 1);
        assert_eq!(pack.active_zone.len(), 1);
        assert_eq!(pack.foundation[0].path, PathBuf::from("AGENTS.md"));
        assert_eq!(
            pack.secondary_skeletons[0].path,
            PathBuf::from("src/lib.rs")
        );
        assert_eq!(pack.active_zone[0].path, PathBuf::from("focus.py"));
        assert_eq!(pack.active_zone[0].body, "print('focus')\r\n");
    }

    #[test]
    fn normalizes_crlf_and_cr_before_rendering() {
        let selection = PackSelection {
            foundation: vec![],
            secondary_skeletons: vec![],
            active_zone: vec![PackFile::new("focus.txt", Some("text"), "a\r\nb\rc\n")],
        };

        assert!(render_markdown(&selection).contains("a\nb\nc\n"));
    }

    #[test]
    fn escalates_fence_from_normalized_body() {
        let selection = PackSelection {
            foundation: vec![],
            secondary_skeletons: vec![],
            active_zone: vec![PackFile::new(
                "focus.md",
                Some("markdown"),
                "line\r\n```\n```` body\n",
            )],
        };

        assert!(render_markdown(&selection).contains("`````markdown\n"));
    }

    #[test]
    fn active_zone_renders_raw_full_body() {
        let body = "def f():\n    return \"raw body stays\"\n";
        let selection = PackSelection {
            foundation: vec![],
            secondary_skeletons: vec![],
            active_zone: vec![PackFile::new("focus.py", Some("python"), body)],
        };

        assert!(render_markdown(&selection).contains(body));
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
            "frugal-pack-{label}-{}-{}",
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
