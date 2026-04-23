pub mod app;
pub mod cli;
pub mod config;
pub mod discovery;
pub mod error;
pub mod init;
pub mod languages;
pub mod markers;
pub mod pack;
pub mod status;
pub mod token;

pub use app::{App, InitCommand, PackCommand, StatusCommand, StubRunner};
pub use cli::{Cli, Command};
pub use error::{Error, Result};
pub use init::InitRunner;
pub use pack::PackRunner;
pub use status::StatusRunner;

pub fn run() -> Result<()> {
    App::new(InitRunner, PackRunner, StatusRunner).run(Cli::parse())
}
