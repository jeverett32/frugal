use crate::app::GainCommand;
use crate::cli::GainArgs;
use crate::discovery::Selection;
use crate::error::{Error, Result};
use crate::token::estimate_tokens;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const STATE_DIR: &str = ".fgl";
const HISTORY_PATH: &str = ".fgl/history.jsonl";

#[derive(Debug, Default, Clone, Copy)]
pub struct GainRunner;

impl GainCommand for GainRunner {
    fn run(&self, args: &GainArgs) -> Result<()> {
        let _ = args;
        let cwd = std::env::current_dir().map_err(Error::io)?;
        let summary = GainSummary::from_events(&load_history(&cwd)?);

        let mut stdout = io::stdout().lock();
        writeln!(stdout, "{summary}").map_err(Error::io)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GainEvent {
    pub timestamp_unix: u64,
    pub files: usize,
    pub languages: usize,
    pub active_files: Vec<String>,
    pub raw_tokens: usize,
    pub pack_tokens: usize,
    pub prefix_tokens: usize,
    pub active_tokens: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GainSummary {
    pub packs: usize,
    pub raw_tokens: usize,
    pub pack_tokens: usize,
    pub saved_tokens: i64,
    pub prefix_tokens: usize,
    pub active_tokens: usize,
}

impl GainSummary {
    pub fn from_events(events: &[GainEvent]) -> Self {
        let raw_tokens = events.iter().map(|event| event.raw_tokens).sum::<usize>();
        let pack_tokens = events.iter().map(|event| event.pack_tokens).sum::<usize>();

        Self {
            packs: events.len(),
            raw_tokens,
            pack_tokens,
            saved_tokens: raw_tokens as i64 - pack_tokens as i64,
            prefix_tokens: events.iter().map(|event| event.prefix_tokens).sum(),
            active_tokens: events.iter().map(|event| event.active_tokens).sum(),
        }
    }

    fn savings_percent(&self) -> String {
        if self.raw_tokens == 0 {
            "0.00".to_string()
        } else {
            format!(
                "{:.2}",
                (self.saved_tokens as f64 / self.raw_tokens as f64) * 100.0
            )
        }
    }
}

impl std::fmt::Display for GainSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "packs={} raw={} pack={} saved={} savings={} prefix={} active={}",
            self.packs,
            self.raw_tokens,
            self.pack_tokens,
            self.saved_tokens,
            self.savings_percent(),
            self.prefix_tokens,
            self.active_tokens,
        )
    }
}

pub fn append_pack_history(repo_root: &Path, selection: &Selection, rendered: &str) -> Result<()> {
    let state_dir = repo_root.join(STATE_DIR);
    if !state_dir.is_dir() {
        return Ok(());
    }

    let event = build_gain_event(selection, rendered)?;
    let line = serde_json::to_string(&event)
        .map_err(|error| Error::history(format!("failed to serialize gain event: {error}")))?;

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(repo_root.join(HISTORY_PATH))
        .map_err(Error::io)?;
    writeln!(file, "{line}").map_err(Error::io)
}

pub fn load_history(repo_root: &Path) -> Result<Vec<GainEvent>> {
    let path = repo_root.join(HISTORY_PATH);
    let file = match fs::File::open(&path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(Error::io(error)),
    };

    let reader = io::BufReader::new(file);
    let mut events = Vec::new();
    for (index, line) in reader.lines().enumerate() {
        let line = line.map_err(Error::io)?;
        if line.trim().is_empty() {
            continue;
        }
        let event = serde_json::from_str::<GainEvent>(&line).map_err(|error| {
            Error::history(format!(
                "invalid history entry at line {}: {error}",
                index + 1
            ))
        })?;
        events.push(event);
    }

    Ok(events)
}

fn build_gain_event(selection: &Selection, rendered: &str) -> Result<GainEvent> {
    let prefix_bytes = selection
        .foundation
        .iter()
        .chain(selection.secondary.iter())
        .map(|path| {
            fs::read(&path.absolute_path)
                .map(|bytes| bytes.len())
                .map_err(Error::io)
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .sum::<usize>();
    let active_bytes = selection
        .active
        .iter()
        .map(|path| {
            fs::read(&path.absolute_path)
                .map(|bytes| bytes.len())
                .map_err(Error::io)
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .sum::<usize>();
    let prefix_pack_bytes = rendered
        .split("# Active Zone")
        .next()
        .unwrap_or(rendered)
        .len();

    Ok(GainEvent {
        timestamp_unix: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock before unix epoch")
            .as_secs(),
        files: selection.foundation.len() + selection.secondary.len() + selection.active.len(),
        languages: selection
            .foundation
            .iter()
            .chain(selection.secondary.iter())
            .chain(selection.active.iter())
            .filter_map(|path| path.language.label())
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        active_files: selection
            .active
            .iter()
            .map(|path| path.repo_relative_path.to_string_lossy().into_owned())
            .collect(),
        raw_tokens: estimate_tokens(prefix_bytes + active_bytes),
        pack_tokens: estimate_tokens(rendered.len()),
        prefix_tokens: estimate_tokens(prefix_pack_bytes),
        active_tokens: estimate_tokens(active_bytes),
    })
}

#[cfg(test)]
mod tests {
    use super::{GainEvent, GainSummary};

    #[test]
    fn summary_formats_exact_one_line_shape() {
        let summary = GainSummary::from_events(&[
            GainEvent {
                timestamp_unix: 1,
                files: 4,
                languages: 2,
                active_files: vec!["docs/a.md".into()],
                raw_tokens: 20,
                pack_tokens: 8,
                prefix_tokens: 6,
                active_tokens: 2,
            },
            GainEvent {
                timestamp_unix: 2,
                files: 5,
                languages: 3,
                active_files: vec!["docs/b.md".into()],
                raw_tokens: 10,
                pack_tokens: 5,
                prefix_tokens: 4,
                active_tokens: 1,
            },
        ]);

        assert_eq!(
            summary.to_string(),
            "packs=2 raw=30 pack=13 saved=17 savings=56.67 prefix=10 active=3"
        );
    }

    #[test]
    fn summary_handles_empty_history() {
        let summary = GainSummary::from_events(&[]);

        assert_eq!(
            summary.to_string(),
            "packs=0 raw=0 pack=0 saved=0 savings=0.00 prefix=0 active=0"
        );
    }
}
