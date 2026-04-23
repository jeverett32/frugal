use crate::app::InitCommand;
use crate::cli::InitArgs;
use crate::config::Config;
use crate::error::{Error, Result};
use crate::markers::upsert_managed_block;
use std::fs;
use std::path::Path;

const CONFIG_DIR: &str = ".fgl";
const CONFIG_PATH: &str = ".fgl/config.toml";
const AGENTS_PATH: &str = "AGENTS.md";
const CLAUDE_PATH: &str = "CLAUDE.md";

const AGENTS_BODY: &str =
    "# AGENTS\n\n- Read `AGENTS.md` before changing code.\n- Keep patches small and deterministic.\n";
const CLAUDE_BODY: &str =
    "# CLAUDE\n\n- Read `AGENTS.md` before changing code.\n- Keep patches small and deterministic.\n";

#[derive(Debug, Default, Clone, Copy)]
pub struct InitRunner;

impl InitCommand for InitRunner {
    fn run(&self, args: &InitArgs) -> Result<()> {
        let _ = args;
        init_repo(Path::new("."))
    }
}

fn init_repo(repo_root: &Path) -> Result<()> {
    fs::create_dir_all(repo_root.join(CONFIG_DIR)).map_err(Error::io)?;
    ensure_config(repo_root)?;
    ensure_managed_doc(repo_root, AGENTS_PATH, AGENTS_BODY)?;
    ensure_managed_doc(repo_root, CLAUDE_PATH, CLAUDE_BODY)?;
    Ok(())
}

fn ensure_config(repo_root: &Path) -> Result<()> {
    if load_config(repo_root)?.is_some() {
        return Ok(());
    }

    let mut config = Config::default();
    config.foundation.pinned_paths = vec![AGENTS_PATH.to_string(), CLAUDE_PATH.to_string()];

    let rendered = config.render().map_err(Error::config)?;
    fs::write(repo_root.join(CONFIG_PATH), rendered).map_err(Error::io)
}

pub fn load_config(repo_root: &Path) -> Result<Option<Config>> {
    let path = repo_root.join(CONFIG_PATH);
    match fs::read_to_string(&path) {
        Ok(contents) => Config::parse(&contents).map(Some).map_err(Error::config),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(Error::io(error)),
    }
}

fn ensure_managed_doc(repo_root: &Path, relative_path: &str, body: &str) -> Result<()> {
    let path = repo_root.join(relative_path);
    let input = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(Error::io(error)),
    };

    let output = upsert_managed_block(&input, body).map_err(Error::marker)?;
    if output != input {
        fs::write(path, output).map_err(Error::io)?;
    }

    Ok(())
}
