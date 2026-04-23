use clap::{Args, CommandFactory, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "fgl",
    version,
    about = "Predictable prompt-pack assembly and repo bootstrap CLI",
    arg_required_else_help = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, Subcommand, PartialEq, Eq)]
pub enum Command {
    /// Bootstrap repo metadata and managed docs
    Init(InitArgs),
    /// Assemble prompt-pack markdown
    Pack(PackArgs),
    /// Print one-line prompt-pack metrics
    Status(StatusArgs),
    /// Summarize estimated token savings from pack history
    Gain(GainArgs),
}

#[derive(Debug, Clone, Args, PartialEq, Eq, Default)]
pub struct InitArgs {
    /// Rescan repo for languages and update config
    #[arg(long)]
    pub rescan: bool,
}

#[derive(Debug, Clone, Args, PartialEq, Eq, Default)]
pub struct PackArgs {
    /// Write rendered pack to file instead of stdout
    #[arg(short, long, value_name = "PATH")]
    pub output: Option<PathBuf>,

    /// Files to include in Active Zone order
    #[arg(value_name = "PATH")]
    pub paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Args, PartialEq, Eq, Default)]
pub struct StatusArgs {
    /// Files to include in Active Zone order
    #[arg(value_name = "PATH")]
    pub paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Args, PartialEq, Eq)]
pub struct GainArgs {
    /// Emit machine-readable JSON
    #[arg(long)]
    pub json: bool,

    /// Show recent pack runs
    #[arg(long)]
    pub history: bool,

    /// Maximum recent runs to show
    #[arg(long, value_name = "N", default_value_t = 10)]
    pub limit: usize,

    /// Aggregate savings across all registered repos
    #[arg(long)]
    pub global: bool,
}

impl Default for GainArgs {
    fn default() -> Self {
        Self {
            json: false,
            history: false,
            limit: 10,
            global: false,
        }
    }
}

impl Cli {
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }

    pub fn command_for_tests() -> clap::Command {
        <Self as CommandFactory>::command()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{error::ErrorKind, Parser};

    #[test]
    fn clap_shape_is_valid() {
        Cli::command_for_tests().debug_assert();
    }

    #[test]
    fn parse_pack_paths() {
        let cli = Cli::try_parse_from(["fgl", "pack", "a.md", "b.md"]).expect("pack parses");

        assert_eq!(
            cli.command,
            Command::Pack(PackArgs {
                output: None,
                paths: vec![PathBuf::from("a.md"), PathBuf::from("b.md")],
            })
        );
    }

    #[test]
    fn parse_pack_output_flag() {
        let cli = Cli::try_parse_from(["fgl", "pack", "--output", "CONTEXT.md", "a.md"])
            .expect("pack parses");

        assert_eq!(
            cli.command,
            Command::Pack(PackArgs {
                output: Some(PathBuf::from("CONTEXT.md")),
                paths: vec![PathBuf::from("a.md")],
            })
        );
    }

    #[test]
    fn root_requires_subcommand() {
        let error = Cli::try_parse_from(["fgl"]).expect_err("missing subcommand");

        assert_eq!(
            error.kind(),
            ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
        );
    }

    #[test]
    fn init_help_renders() {
        let error = Cli::try_parse_from(["fgl", "init", "--help"]).expect_err("help exits");

        assert_eq!(error.kind(), ErrorKind::DisplayHelp);
    }

    #[test]
    fn parse_gain_subcommand() {
        let cli = Cli::try_parse_from(["fgl", "gain"]).expect("gain parses");

        assert_eq!(cli.command, Command::Gain(GainArgs::default()));
    }

    #[test]
    fn parse_gain_flags() {
        let cli = Cli::try_parse_from(["fgl", "gain", "--json", "--history", "--limit", "5"])
            .expect("gain flags parse");

        assert_eq!(
            cli.command,
            Command::Gain(GainArgs {
                json: true,
                history: true,
                limit: 5,
                global: false,
            })
        );
    }
}
