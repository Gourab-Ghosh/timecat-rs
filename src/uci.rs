use super::*;

trait IntoSpin {
    fn into_spin(self) -> Spin;
}

macro_rules! impl_into_spin {
    ($type_:ty) => {
        impl IntoSpin for $type_ {
            fn into_spin(self) -> Spin {
                self as Spin
            }
        }
    };

    ($type_:ty, $func:ident) => {
        impl IntoSpin for $type_ {
            fn into_spin(self) -> Spin {
                self.$func() as Spin
            }
        }
    };
}

impl_into_spin!(usize);
impl_into_spin!(CacheTableSize, unwrap);
impl_into_spin!(Duration, as_millis);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum UCIOptionType {
    Button {
        function: fn(&mut Engine),
    },
    Check {
        default: bool,
        function: fn(&mut Engine, bool),
    },
    String {
        default: String,
        function: fn(&mut Engine, &str),
    },
    Spin {
        default: Spin,
        min: Spin,
        max: Spin,
        function: fn(&mut Engine, Spin),
    },
    Combo {
        default: String,
        options: Vec<String>,
        function: fn(&mut Engine, &str),
    },
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq)]
struct SpinValue<T: Clone + Copy + IntoSpin> {
    default: T,
    min: T,
    max: T,
}

impl<T: Clone + Copy + IntoSpin> SpinValue<T> {
    #[inline]
    pub const fn new(default: T, min: T, max: T) -> Self {
        Self { default, min, max }
    }

    #[inline]
    pub const fn get_default(self) -> T {
        self.default
    }

    #[inline]
    pub const fn get_min(self) -> T {
        self.min
    }

    #[inline]
    pub const fn get_max(self) -> T {
        self.max
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct UCIOption {
    name: String,
    sorted_alias: Vec<String>,
    option_type: UCIOptionType,
}

impl UCIOption {
    fn new(name: &str, option_type: UCIOptionType) -> Self {
        Self {
            name: name.trim().to_string(),
            sorted_alias: vec![],
            option_type,
        }
    }

    fn alias(mut self, name: &str) -> Self {
        self.sorted_alias.push(name.trim().to_lowercase());
        self.sorted_alias.sort_unstable();
        self
    }

    fn new_spin<T: Clone + Copy + IntoSpin>(
        name: &str,
        values: SpinValue<T>,
        function: fn(&mut Engine, Spin),
    ) -> Self {
        UCIOption::new(
            name,
            UCIOptionType::Spin {
                default: values.get_default().into_spin(),
                min: values.get_min().into_spin(),
                max: values.get_max().into_spin(),
                function,
            },
        )
    }

    fn new_check(name: &str, default: bool, function: fn(&mut Engine, bool)) -> Self {
        UCIOption::new(name, UCIOptionType::Check { default, function })
    }

    fn new_button(name: &str, function: fn(&mut Engine)) -> Self {
        UCIOption::new(name, UCIOptionType::Button { function })
    }

    fn set_option(&self, engine: &mut Engine, value_string: String) -> Result<()> {
        match self.option_type {
            UCIOptionType::Check { function, .. } => {
                function(engine, value_string.parse()?);
            }
            UCIOptionType::Spin {
                min, max, function, ..
            } => {
                let value = value_string.parse()?;
                if value < min || value > max {
                    return Err(TimecatError::InvalidSpinValue {
                        name: self.name.to_owned(),
                        value,
                        min,
                        max,
                    });
                }
                function(engine, value);
            }
            UCIOptionType::Combo { function, .. } => {
                function(engine, &value_string);
            }
            UCIOptionType::Button { function } => {
                function(engine);
            }
            UCIOptionType::String { function, .. } => {
                function(engine, &value_string);
            }
        }
        Ok(())
    }
}

impl fmt::Display for UCIOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match &self.option_type {
            UCIOptionType::Check { default, .. } => {
                format!(
                    "{} {} {} {}",
                    "option name".colorize(INFO_MESSAGE_STYLE),
                    self.name,
                    "type check default".colorize(INFO_MESSAGE_STYLE),
                    default,
                )
            }
            UCIOptionType::Spin {
                default, min, max, ..
            } => {
                format!(
                    "{} {} {} {} {} {} {} {}",
                    "option name".colorize(INFO_MESSAGE_STYLE),
                    self.name,
                    "type spin default".colorize(INFO_MESSAGE_STYLE),
                    default,
                    "min".colorize(INFO_MESSAGE_STYLE),
                    min,
                    "max".colorize(INFO_MESSAGE_STYLE),
                    max,
                )
            }
            UCIOptionType::Combo {
                default, options, ..
            } => {
                format!(
                    "{} {} {} {} {}",
                    "option name".colorize(INFO_MESSAGE_STYLE),
                    self.name,
                    "type combo default".colorize(INFO_MESSAGE_STYLE),
                    default,
                    options.iter().map(|s| format!("var {s}")).join(" "),
                )
            }
            UCIOptionType::Button { .. } => {
                format!(
                    "{} {} {}",
                    "option name".colorize(INFO_MESSAGE_STYLE),
                    self.name,
                    "type button".colorize(INFO_MESSAGE_STYLE),
                )
            }
            UCIOptionType::String { default, .. } => {
                format!(
                    "{} {} {} {}",
                    "option name".colorize(INFO_MESSAGE_STYLE),
                    self.name,
                    "type string default".colorize(INFO_MESSAGE_STYLE),
                    default,
                )
            }
        };
        write!(f, "{s}")
    }
}

pub struct UCIStateManager {
    options: RwLock<Vec<UCIOption>>,
}

impl UCIStateManager {
    pub const fn dummy() -> Self {
        Self {
            options: RwLock::new(Vec::new()),
        }
    }

    fn new() -> Self {
        Self {
            options: RwLock::new(get_uci_state_manager()),
        }
    }

    pub fn get_option(&self, command_name: &str) -> Option<UCIOption> {
        let command_name = command_name.to_string();
        self.options
            .read()
            .unwrap()
            .iter()
            .find(
                |UCIOption {
                     name, sorted_alias, ..
                 }| {
                    name.eq_ignore_ascii_case(&command_name)
                        || sorted_alias.binary_search(&command_name).is_ok()
                },
            )
            .cloned()
    }

    pub fn get_all_options(&self) -> Vec<UCIOption> {
        self.options.read().unwrap().to_owned()
    }

    pub fn run_command(&self, engine: &mut Engine, user_input: &str) -> Result<()> {
        let binding = Parser::sanitize_string(user_input);
        let commands = binding.split_whitespace().collect_vec();
        if commands
            .first()
            .ok_or(TimecatError::UnknownCommand)?
            .to_lowercase()
            != "setoption"
        {
            return Err(TimecatError::UnknownCommand);
        }
        if commands
            .get(1)
            .ok_or(TimecatError::UnknownCommand)?
            .to_lowercase()
            != "name"
        {
            return Err(TimecatError::UnknownCommand);
        }
        let command_name = commands
            .iter()
            .skip(2)
            .take_while(|&&c| c != "value")
            .join(" ")
            .to_lowercase();
        let value_string = commands
            .iter()
            .skip_while(|&&s| s != "value")
            .skip(1)
            .join(" ");

        self.get_option(&command_name)
            .ok_or(TimecatError::UnknownCommand)?
            .set_option(engine, value_string)
    }
}

impl Default for UCIStateManager {
    fn default() -> Self {
        Self::new()
    }
}

fn get_uci_state_manager() -> Vec<UCIOption> {
    let t_table_size_uci = SpinValue::new(
        TIMECAT_DEFAULTS.t_table_size,
        CacheTableSize::Exact(1),
        CacheTableSize::Exact({
            let transposition_table_entry_size =
                CacheTableSize::get_entry_size::<TranspositionTableEntry>();
            let evaluator_entry_size = CacheTableSize::get_entry_size::<Score>();
            let max_size = if transposition_table_entry_size > evaluator_entry_size {
                transposition_table_entry_size
            } else {
                evaluator_entry_size
            };
            (usize::MAX >> 20) / max_size
        }),
    );

    let move_overhead_uci = SpinValue::new(
        TIMECAT_DEFAULTS.move_overhead,
        Duration::from_secs(0),
        Duration::MAX,
    );

    let options = vec![
        UCIOption::new_spin(
            "Threads",
            SpinValue::new(TIMECAT_DEFAULTS.num_threads.get(), 1, 1024),
            |engine, value| {
                let num_threads = unsafe { NonZeroUsize::new_unchecked(value as usize) };
                engine.set_num_threads(num_threads);
                print_uci_info("Number of threads is set to", num_threads);
            },
        )
        .alias("Thread"),
        UCIOption::new_spin("Hash", t_table_size_uci, {
            |engine, value| {
                let size = CacheTableSize::Exact(value as usize);
                let transposition_table = engine.get_transposition_table();
                transposition_table.set_size(size);
                if GLOBAL_TIMECAT_STATE.is_in_debug_mode() {
                    transposition_table.print_info();
                }
                print_uci_info(
                    "Transposition table is set to size to",
                    size.to_cache_table_memory_size::<TranspositionTableEntry>(),
                );
            }
        }),
        UCIOption::new_button("Clear Hash", |engine| {
            engine.get_transposition_table().clear();
            engine.get_board().get_evaluator().clear();
            print_uci_info::<&str>("All hash tables are cleared!", None);
        }),
        UCIOption::new_spin("Move Overhead", move_overhead_uci, |engine, value| {
            let duration = Duration::from_millis(value as u64);
            engine.set_move_overhead(duration);
            print_uci_info("Move Overhead is set to", duration.stringify());
        }),
        // UCIOption::new_check(
        //     "OwnBook",
        //     TIMECAT_DEFAULTS.use_own_book,
        //     |engine, b| {
        //         use_own_book.store(b, MEMORY_ORDERING);
        //         print_uci_info("Own Book Usage is set to", b);
        //     },
        // ),
        // UCIOption::new_check(
        //     "UCI_Chess960",
        //     TIMECAT_DEFAULTS.chess960_mode,
        //     |engine, b| {
        //         chess960_mode.store(b, MEMORY_ORDERING);
        //         print_uci_info("Chess 960 mode is set to", b);
        //     },
        // ),
    ];
    options
}
