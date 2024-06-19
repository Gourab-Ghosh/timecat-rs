use super::*;

#[derive(Clone, Debug)]
enum TimecatBuilderAction {
    PrintHelpCommand,
    PrintEngineVersion,
    #[cfg(feature = "debug")]
    RunTest,
    RunCommand(String),
}

#[derive(Default)]
pub struct TimecatBuilder {
    actions: Vec<TimecatBuilderAction>,
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
            uci_options: UCIStateManager::default(),
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
            self.actions.push(TimecatBuilderAction::PrintHelpCommand);
            return self;
        }
        if args.contains(&"--version") {
            self.actions.push(TimecatBuilderAction::PrintEngineVersion);
            return self;
        }
        #[cfg(feature = "debug")]
        if args.contains(&"--test") {
            self.actions.push(TimecatBuilderAction::RunTest);
            return self;
        }
        if args.contains(&"-c") || args.contains(&"--command") {
            let command = args
                .iter()
                .skip_while(|&arg| !["-c", "--command"].contains(arg))
                .skip(1)
                .take_while(|&&arg| !arg.starts_with("--"))
                .join(" ");
            self.actions.push(TimecatBuilderAction::RunCommand(command));
            return self;
        }
        self
    }
}

pub struct Timecat {
    actions: Vec<TimecatBuilderAction>,
    engine: Engine,
    io_reader: IoReader,
    uci_options: UCIStateManager,
}

impl Timecat {
    pub fn run(mut self) {
        self.io_reader.start_reader();
        #[allow(clippy::never_loop)]
        for action in self.actions.clone().into_iter() {
            match action {
                TimecatBuilderAction::PrintHelpCommand => {
                    println!("{}", UserCommand::generate_help_message());
                    return;
                }
                TimecatBuilderAction::PrintEngineVersion => {
                    print_engine_version(false);
                    return;
                }
                #[cfg(feature = "debug")]
                TimecatBuilderAction::RunTest => {
                    test.run_and_print_time(&mut self.engine).unwrap();
                    return;
                }
                TimecatBuilderAction::RunCommand(user_input) => {
                    println!();
                    self.run_uci_command(&user_input).unwrap();
                    return;
                }
            }
        }
        print_engine_info(self.engine.get_transposition_table());
        // self.main_loop.run_and_print_time()?;
        self.main_loop();
    }

    fn print_exit_message() {
        if GLOBAL_UCI_STATE.is_in_console_mode() {
            println!(
                "{}",
                "Program ended successfully!".colorize(SUCCESS_MESSAGE_STYLE)
            );
        }
    }

    fn get_input<T: fmt::Display>(q: T, io_reader: &IoReader) -> String {
        print_line(q);
        io_reader.read_line()
    }

    pub fn run_uci_command(&mut self, raw_input: &str) -> Result<()> {
        for user_command in Parser::parse_command(raw_input)? {
            user_command.run_command(&mut self.engine, &self.uci_options)?;
        }
        Ok(())
    }

    pub fn main_loop(&mut self) {
        loop {
            if GLOBAL_UCI_STATE.terminate_engine() {
                Self::print_exit_message();
                break;
            }
            let raw_input = if GLOBAL_UCI_STATE.is_in_console_mode() {
                println!();
                let raw_input = Self::get_input(
                    "Enter Command: ".colorize(INPUT_MESSAGE_STYLE),
                    &self.io_reader,
                );
                println!();
                raw_input
            } else {
                Self::get_input("", &self.io_reader)
            };
            self.run_uci_command(&raw_input).unwrap_or_else(|error| {
                println!(
                    "{}",
                    error
                        .stringify_with_optional_raw_input(Some(raw_input.as_str()))
                        .colorize(ERROR_MESSAGE_STYLE)
                )
            });
        }
    }

    pub fn uci_loop(&mut self) {
        GLOBAL_UCI_STATE.set_to_uci_mode();
        self.main_loop();
    }
}
