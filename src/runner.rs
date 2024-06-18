use super::*;

enum Action {
    PrintHelpCommand,
    PrintEngineVersion,
    #[cfg(feature = "debug")]
    RunTest,
    RunCommand(String),
}

#[derive(Default)]
pub struct TimecatBuilder {
    actions: Vec<Action>,
    engine: Option<Engine>,
}

impl TimecatBuilder {
    pub fn build(self) -> Timecat {
        let io_reader = IoReader::default();
        Timecat {
            actions: self.actions,
            engine: self
                .engine
                .unwrap_or_default()
                .with_io_reader(io_reader.clone()),
            io_reader,
        }
    }

    pub fn parse_args(mut self, args: &[&str]) -> Self {
        if args.contains(&"--uci") {
            GLOBAL_UCI_STATE.set_to_uci_mode();
        }
        #[cfg(feature = "colored_output")]
        if args.contains(&"--no-color") {
            GLOBAL_UCI_STATE.set_colored_output(false, false);
        }
        if args.contains(&"--threads") {
            let num_threads = args
                .iter()
                .skip_while(|&arg| !arg.starts_with("--threads"))
                .nth(1)
                .unwrap_or(&"")
                .parse()
                .unwrap_or(GlobalUCIState::default().get_num_threads());
            GLOBAL_UCI_STATE.set_num_threads(num_threads, false);
        }
        if args.contains(&"--help") {
            self.actions.push(Action::PrintHelpCommand);
            return self;
        }
        if args.contains(&"--version") {
            self.actions.push(Action::PrintEngineVersion);
            return self;
        }
        #[cfg(feature = "debug")]
        if args.contains(&"--test") {
            self.actions.push(Action::RunTest);
            return self;
        }
        if args.contains(&"-c") || args.contains(&"--command") {
            let command = args
                .iter()
                .skip_while(|&arg| !["-c", "--command"].contains(arg))
                .skip(1)
                .take_while(|&&arg| !arg.starts_with("--"))
                .join(" ");
            self.actions.push(Action::RunCommand(command));
            return self;
        }
        self
    }
}

pub struct Timecat {
    actions: Vec<Action>,
    engine: Engine,
    io_reader: IoReader,
}

impl Timecat {
    pub fn run(mut self) -> Result<(), TimecatError> {
        self.io_reader.start_reader();
        #[allow(clippy::never_loop)]
        for action in self.actions.into_iter() {
            match action {
                Action::PrintHelpCommand => {
                    println!("{}", Parser::get_help_message());
                    return Ok(());
                }
                Action::PrintEngineVersion => {
                    print_engine_version(false);
                    return Ok(());
                }
                #[cfg(feature = "debug")]
                Action::RunTest => {
                    test.run_and_print_time(&mut self.engine)?;
                    return Ok(());
                }
                Action::RunCommand(command) => {
                    println!();
                    Parser::run_raw_input_checked(&mut self.engine, &command);
                    return Ok(());
                }
            }
        }
        print_engine_info(self.engine.get_transposition_table());
        Parser::main_loop.run_and_print_time(&mut self.engine, &self.io_reader);
        Ok(())
    }
}
