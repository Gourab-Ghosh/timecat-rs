use super::*;

#[derive(Default)]
pub struct TimecatBuilder {
    user_commands: Vec<UserCommand>,
    engine: Option<Engine>,
}

impl TimecatBuilder {
    pub fn build(self) -> Timecat {
        let io_reader = IoReader::default();
        Timecat {
            user_commands: self.user_commands,
            engine: self
                .engine
                .unwrap_or_default()
                .with_io_reader(io_reader.clone()),
            io_reader,
            uci_state_manager: UCIStateManager::default(),
        }
    }

    pub fn parse_args(mut self, args: &[&str]) -> Self {
        if args.contains(&"--uci") {
            self.user_commands
                .push(UserCommand::ChangeToUCIMode { verbose: false });
        }
        #[cfg(feature = "colored")]
        if args.contains(&"--no-color") {
            self.user_commands.push(UserCommand::SetColor(false));
        }
        if args.contains(&"--threads") {
            let num_threads = args
                .iter()
                .skip_while(|&arg| !arg.starts_with("--threads"))
                .nth(1)
                .unwrap_or(&"")
                .parse()
                .unwrap_or(TIMECAT_DEFAULTS.num_threads);
            self.user_commands.push(UserCommand::SetUCIOption {
                user_input: format!("setoption name Threads value {}", num_threads),
            });
        }
        if args.contains(&"--help") {
            self.user_commands.push(UserCommand::Help);
            self.user_commands.push(UserCommand::TerminateEngine);
            return self;
        }
        if args.contains(&"--version") {
            self.user_commands.push(UserCommand::EngineVersion);
            self.user_commands.push(UserCommand::TerminateEngine);
            return self;
        }
        #[cfg(feature = "debug")]
        if args.contains(&"--test") {
            self.user_commands.push(UserCommand::RunTest);
            self.user_commands.push(UserCommand::TerminateEngine);
            return self;
        }
        if args.contains(&"-c") || args.contains(&"--command") {
            let command_string = args
                .iter()
                .skip_while(|&arg| !["-c", "--command"].contains(arg))
                .skip(1)
                .take_while(|&&arg| !arg.starts_with("--"))
                .join(" ");
            match Parser::parse_command(&command_string) {
                Ok(user_commands) => self.user_commands.extend(user_commands),
                Err(error) => println_wasm!(
                    "{}",
                    error
                        .stringify_with_optional_raw_input(Some(&command_string))
                        .colorize(ERROR_MESSAGE_STYLE)
                ),
            }
            self.user_commands.push(UserCommand::TerminateEngine);
            return self;
        }
        self
    }
}

pub struct Timecat {
    user_commands: Vec<UserCommand>,
    engine: Engine,
    io_reader: IoReader,
    uci_state_manager: UCIStateManager,
}

impl Timecat {
    pub fn run(mut self) {
        self.io_reader.start_reader_in_parallel();
        for user_command in self.user_commands.iter() {
            user_command
                .run_command(&mut self.engine, &self.uci_state_manager)
                .unwrap_or_else(|error| {
                    println_wasm!("{}", error.stringify().colorize(ERROR_MESSAGE_STYLE))
                });
        }
        if self.engine.terminate() {
            return;
        }
        print_engine_info(
            self.engine.get_transposition_table(),
            self.engine.get_board().get_evaluator(),
        );
        Self::main_loop.run_and_print_time(&mut self);
    }

    pub fn run_uci_command(&mut self, raw_input: &str) -> Result<()> {
        for user_command in Parser::parse_command(raw_input)? {
            user_command.run_command(&mut self.engine, &self.uci_state_manager)?;
        }
        Ok(())
    }

    pub fn main_loop(&mut self) {
        loop {
            if self.engine.terminate() {
                if GLOBAL_TIMECAT_STATE.is_in_console_mode() {
                    println_wasm!(
                        "{}",
                        "Program ended successfully!".colorize(SUCCESS_MESSAGE_STYLE)
                    );
                }
                break;
            }
            let raw_input = if GLOBAL_TIMECAT_STATE.is_in_console_mode() {
                println_wasm!();
                let raw_input = get_input(
                    "Enter Command: ".colorize(INPUT_MESSAGE_STYLE),
                    &self.io_reader,
                );
                println_wasm!();
                raw_input
            } else {
                get_input("", &self.io_reader)
            };
            self.run_uci_command(&raw_input).unwrap_or_else(|error| {
                println_wasm!(
                    "{}",
                    error
                        .stringify_with_optional_raw_input(Some(raw_input.as_str()))
                        .colorize(ERROR_MESSAGE_STYLE)
                )
            });
        }
    }

    pub fn uci_loop(&mut self) {
        GLOBAL_TIMECAT_STATE.set_to_uci_mode();
        self.main_loop();
    }
}
