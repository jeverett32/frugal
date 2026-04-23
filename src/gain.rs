use crate::app::GainCommand;
use crate::cli::GainArgs;
use crate::discovery::Selection;
use crate::error::{Error, Result};
use crate::token::estimate_tokens;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};
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
        let cwd = std::env::current_dir().map_err(Error::io)?;
        let events = load_history(&cwd)?;
        let report = GainReport::from_events(&events, args.limit);

        let mut stdout = io::stdout().lock();
        if args.json {
            let json = serde_json::to_string_pretty(&report).map_err(|error| {
                Error::history(format!("failed to serialize gain report: {error}"))
            })?;
            writeln!(stdout, "{json}").map_err(Error::io)
        } else if args.history {
            write!(stdout, "{}", render_history_report(&report)).map_err(Error::io)
        } else {
            write!(stdout, "{}", render_summary_report(&report)).map_err(Error::io)
        }
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GainSummary {
    pub packs: usize,
    pub raw_tokens: usize,
    pub pack_tokens: usize,
    pub saved_tokens: i64,
    pub prefix_tokens: usize,
    pub active_tokens: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveFileStat {
    pub path: String,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecentRun {
    pub timestamp_unix: u64,
    pub active_files: Vec<String>,
    pub raw_tokens: usize,
    pub pack_tokens: usize,
    pub saved_tokens: i64,
    pub savings_percent: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GainReport {
    pub summary: GainSummary,
    pub top_active_files: Vec<ActiveFileStat>,
    pub recent_runs: Vec<RecentRun>,
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

    pub fn savings_percent(&self) -> String {
        format_percent(self.saved_tokens, self.raw_tokens)
    }
}

impl GainReport {
    pub fn from_events(events: &[GainEvent], limit: usize) -> Self {
        let mut counts = HashMap::<String, usize>::new();
        for event in events {
            for path in &event.active_files {
                *counts.entry(path.clone()).or_default() += 1;
            }
        }

        let mut top_active_files = counts
            .into_iter()
            .map(|(path, count)| ActiveFileStat { path, count })
            .collect::<Vec<_>>();
        top_active_files.sort_by(|left, right| {
            right
                .count
                .cmp(&left.count)
                .then_with(|| left.path.cmp(&right.path))
        });
        top_active_files.truncate(limit.min(10));

        let recent_runs = events
            .iter()
            .rev()
            .take(limit)
            .map(|event| RecentRun {
                timestamp_unix: event.timestamp_unix,
                active_files: event.active_files.clone(),
                raw_tokens: event.raw_tokens,
                pack_tokens: event.pack_tokens,
                saved_tokens: event.raw_tokens as i64 - event.pack_tokens as i64,
                savings_percent: format_percent(
                    event.raw_tokens as i64 - event.pack_tokens as i64,
                    event.raw_tokens,
                ),
            })
            .collect::<Vec<_>>();

        Self {
            summary: GainSummary::from_events(events),
            top_active_files,
            recent_runs,
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
        .map(file_size)
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .sum::<usize>();
    let active_bytes = selection
        .active
        .iter()
        .map(file_size)
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
            .collect::<BTreeSet<_>>()
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

fn file_size(path: &crate::discovery::SelectedPath) -> Result<usize> {
    fs::read(&path.absolute_path)
        .map(|bytes| bytes.len())
        .map_err(Error::io)
}

fn format_percent(saved_tokens: i64, raw_tokens: usize) -> String {
    if raw_tokens == 0 {
        "0.00".to_string()
    } else {
        format!("{:.2}", (saved_tokens as f64 / raw_tokens as f64) * 100.0)
    }
}

fn render_summary_report(report: &GainReport) -> String {
    let mut output = String::new();
    output.push_str("FGL Estimated Savings (Repo Scope)\n");
    output.push_str("=================================\n\n");
    output.push_str(&format!("Total packs:      {}\n", report.summary.packs));
    output.push_str(&format!(
        "Raw tokens:       {}\n",
        report.summary.raw_tokens
    ));
    output.push_str(&format!(
        "Pack tokens:      {}\n",
        report.summary.pack_tokens
    ));
    output.push_str(&format!(
        "Tokens saved:     {}\n",
        report.summary.saved_tokens
    ));
    output.push_str(&format!(
        "Savings rate:     {}%\n",
        report.summary.savings_percent()
    ));
    output.push_str(&format!(
        "Prefix tokens:    {}\n",
        report.summary.prefix_tokens
    ));
    output.push_str(&format!(
        "Active tokens:    {}\n",
        report.summary.active_tokens
    ));

    if !report.top_active_files.is_empty() {
        output.push_str("\nTop Active Files\n");
        output.push_str("----------------\n");
        for (index, entry) in report.top_active_files.iter().enumerate() {
            output.push_str(&format!(
                "{}. {} ({})\n",
                index + 1,
                entry.path,
                entry.count
            ));
        }
    }

    output
}

fn render_history_report(report: &GainReport) -> String {
    let mut output = render_summary_report(report);
    output.push_str("\nRecent Runs\n");
    output.push_str("-----------\n");

    if report.recent_runs.is_empty() {
        output.push_str("none\n");
        return output;
    }

    for (index, run) in report.recent_runs.iter().enumerate() {
        let active = if run.active_files.is_empty() {
            "-".to_string()
        } else {
            run.active_files.join(", ")
        };
        output.push_str(&format!(
            "{}. ts={} saved={} ({}%) raw={} pack={} active={}\n",
            index + 1,
            run.timestamp_unix,
            run.saved_tokens,
            run.savings_percent,
            run.raw_tokens,
            run.pack_tokens,
            active,
        ));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::{format_percent, GainEvent, GainReport, GainSummary};

    fn sample_events() -> Vec<GainEvent> {
        vec![
            GainEvent {
                timestamp_unix: 1,
                files: 4,
                languages: 2,
                active_files: vec!["docs/a.md".into(), "src/lib.rs".into()],
                raw_tokens: 20,
                pack_tokens: 8,
                prefix_tokens: 6,
                active_tokens: 2,
            },
            GainEvent {
                timestamp_unix: 2,
                files: 5,
                languages: 3,
                active_files: vec!["docs/a.md".into()],
                raw_tokens: 10,
                pack_tokens: 5,
                prefix_tokens: 4,
                active_tokens: 1,
            },
        ]
    }

    #[test]
    fn summary_formats_exact_one_line_shape() {
        let summary = GainSummary::from_events(&sample_events());

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

    #[test]
    fn report_computes_top_active_files_and_recent_runs() {
        let report = GainReport::from_events(&sample_events(), 10);

        assert_eq!(report.top_active_files[0].path, "docs/a.md");
        assert_eq!(report.top_active_files[0].count, 2);
        assert_eq!(report.recent_runs[0].timestamp_unix, 2);
        assert_eq!(report.recent_runs[0].saved_tokens, 5);
        assert_eq!(report.recent_runs[0].savings_percent, "50.00");
    }

    #[test]
    fn format_percent_handles_zero_raw() {
        assert_eq!(format_percent(0, 0), "0.00");
    }
}
