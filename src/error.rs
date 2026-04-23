use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};
use std::path::PathBuf;

use crate::config::ConfigError;
use crate::markers::MarkerError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    CommandUnavailable {
        command: &'static str,
        detail: &'static str,
    },
    Io(String),
    PathNotFound {
        path: PathBuf,
        origin: PathOrigin,
    },
    PathOutsideRepo {
        path: PathBuf,
        origin: PathOrigin,
    },
    History(String),
    Config(ConfigError),
    Marker(MarkerError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathOrigin {
    Foundation,
    Active,
}

impl Error {
    pub fn command_unavailable(command: &'static str, detail: &'static str) -> Self {
        Self::CommandUnavailable { command, detail }
    }

    pub fn io(error: std::io::Error) -> Self {
        Self::Io(error.to_string())
    }

    pub fn path_not_found(path: PathBuf, origin: PathOrigin) -> Self {
        Self::PathNotFound { path, origin }
    }

    pub fn path_outside_repo(path: PathBuf, origin: PathOrigin) -> Self {
        Self::PathOutsideRepo { path, origin }
    }

    pub fn config(error: ConfigError) -> Self {
        Self::Config(error)
    }

    pub fn history(detail: String) -> Self {
        Self::History(detail)
    }

    pub fn marker(error: MarkerError) -> Self {
        Self::Marker(error)
    }

    pub fn exit_code(&self) -> i32 {
        match self {
            Self::CommandUnavailable { .. } => 2,
            Self::Io(_)
            | Self::PathNotFound { .. }
            | Self::PathOutsideRepo { .. }
            | Self::History(_)
            | Self::Config(_)
            | Self::Marker(_) => 1,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::CommandUnavailable { command, detail } => {
                write!(f, "{command} command unavailable: {detail}")
            }
            Self::Io(detail) => write!(f, "i/o error: {detail}"),
            Self::PathNotFound { path, origin } => {
                write!(f, "{} path not found: {}", origin.label(), path.display())
            }
            Self::PathOutsideRepo { path, origin } => {
                write!(
                    f,
                    "{} path outside repo root: {}",
                    origin.label(),
                    path.display()
                )
            }
            Self::History(detail) => write!(f, "history error: {detail}"),
            Self::Config(error) => write!(f, "config error: {error}"),
            Self::Marker(error) => write!(f, "marker error: {error}"),
        }
    }
}

impl StdError for Error {}

impl PathOrigin {
    fn label(self) -> &'static str {
        match self {
            Self::Foundation => "foundation",
            Self::Active => "active",
        }
    }
}
