use crate::cli::{Cli, Command, GainArgs, InitArgs, PackArgs, StatusArgs};
use crate::error::{Error, Result};

pub trait InitCommand {
    fn run(&self, args: &InitArgs) -> Result<()>;
}

pub trait PackCommand {
    fn run(&self, args: &PackArgs) -> Result<()>;
}

pub trait StatusCommand {
    fn run(&self, args: &StatusArgs) -> Result<()>;
}

pub trait GainCommand {
    fn run(&self, args: &GainArgs) -> Result<()>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct StubRunner;

impl InitCommand for StubRunner {
    fn run(&self, args: &InitArgs) -> Result<()> {
        let _ = args;
        Err(Error::command_unavailable(
            "init",
            "init workflow not wired yet",
        ))
    }
}

impl PackCommand for StubRunner {
    fn run(&self, args: &PackArgs) -> Result<()> {
        let _ = args;
        Err(Error::command_unavailable(
            "pack",
            "pack workflow not wired yet",
        ))
    }
}

impl StatusCommand for StubRunner {
    fn run(&self, args: &StatusArgs) -> Result<()> {
        let _ = args;
        Err(Error::command_unavailable(
            "status",
            "status workflow not wired yet",
        ))
    }
}

impl GainCommand for StubRunner {
    fn run(&self, args: &GainArgs) -> Result<()> {
        let _ = args;
        Err(Error::command_unavailable(
            "gain",
            "gain workflow not wired yet",
        ))
    }
}

#[derive(Debug, Default)]
pub struct App<I = StubRunner, P = StubRunner, S = StubRunner, G = StubRunner> {
    init_runner: I,
    pack_runner: P,
    status_runner: S,
    gain_runner: G,
}

impl<I, P, S, G> App<I, P, S, G> {
    pub fn new(init_runner: I, pack_runner: P, status_runner: S, gain_runner: G) -> Self {
        Self {
            init_runner,
            pack_runner,
            status_runner,
            gain_runner,
        }
    }
}

impl<I: InitCommand, P: PackCommand, S: StatusCommand, G: GainCommand> App<I, P, S, G> {
    pub fn run(&self, cli: Cli) -> Result<()> {
        match cli.command {
            Command::Init(args) => self.init_runner.run(&args),
            Command::Pack(args) => self.pack_runner.run(&args),
            Command::Status(args) => self.status_runner.run(&args),
            Command::Gain(args) => self.gain_runner.run(&args),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[derive(Debug, PartialEq, Eq)]
    enum Seen {
        Init,
        Pack(Vec<String>),
        Status(Vec<String>),
        Gain,
    }

    #[derive(Debug, Default)]
    struct RecordingRunner {
        seen: RefCell<Vec<Seen>>,
    }

    impl InitCommand for RecordingRunner {
        fn run(&self, _args: &InitArgs) -> Result<()> {
            self.seen.borrow_mut().push(Seen::Init);
            Ok(())
        }
    }

    impl PackCommand for RecordingRunner {
        fn run(&self, args: &PackArgs) -> Result<()> {
            self.seen.borrow_mut().push(Seen::Pack(
                args.paths
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect(),
            ));
            Ok(())
        }
    }

    impl StatusCommand for RecordingRunner {
        fn run(&self, args: &StatusArgs) -> Result<()> {
            self.seen.borrow_mut().push(Seen::Status(
                args.paths
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect(),
            ));
            Ok(())
        }
    }

    impl GainCommand for RecordingRunner {
        fn run(&self, _args: &GainArgs) -> Result<()> {
            self.seen.borrow_mut().push(Seen::Gain);
            Ok(())
        }
    }

    #[test]
    fn dispatches_init() {
        let runner = RecordingRunner::default();
        let app = App::new(runner, StubRunner, StubRunner, StubRunner);

        app.run(Cli {
            command: Command::Init(InitArgs::default()),
        })
        .expect("init dispatch");

        assert_eq!(app.init_runner.seen.into_inner(), vec![Seen::Init]);
    }

    #[test]
    fn dispatches_pack() {
        let runner = RecordingRunner::default();
        let app = App::new(StubRunner, runner, StubRunner, StubRunner);

        app.run(Cli {
            command: Command::Pack(PackArgs {
                output: None,
                paths: vec!["active.md".into()],
            }),
        })
        .expect("pack dispatch");

        assert_eq!(
            app.pack_runner.seen.into_inner(),
            vec![Seen::Pack(vec!["active.md".into()])]
        );
    }

    #[test]
    fn dispatches_status() {
        let runner = RecordingRunner::default();
        let app = App::new(StubRunner, StubRunner, runner, StubRunner);

        app.run(Cli {
            command: Command::Status(StatusArgs {
                paths: vec!["focus.md".into()],
            }),
        })
        .expect("status dispatch");

        assert_eq!(
            app.status_runner.seen.into_inner(),
            vec![Seen::Status(vec!["focus.md".into()])]
        );
    }

    #[test]
    fn dispatches_gain() {
        let runner = RecordingRunner::default();
        let app = App::new(StubRunner, StubRunner, StubRunner, runner);

        app.run(Cli {
            command: Command::Gain(GainArgs::default()),
        })
        .expect("gain dispatch");

        assert_eq!(app.gain_runner.seen.into_inner(), vec![Seen::Gain]);
    }
}
