use super::*;

pub trait IntoSpin {
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
        function: fn(&Engine),
    },
    Check {
        default: bool,
        function: fn(&Engine, bool),
    },
    String {
        default: String,
        function: fn(&Engine, &str),
    },
    Spin {
        default: Spin,
        min: Spin,
        max: Spin,
        function: fn(&Engine, Spin),
    },
    Combo {
        default: String,
        options: Vec<String>,
        function: fn(&Engine, &str),
    },
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SpinValue<T: Clone + Copy + IntoSpin> {
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
    alternate_names: Vec<String>,
    option_type: UCIOptionType,
}

impl UCIOption {
    fn new(name: &str, option_type: UCIOptionType) -> Self {
        Self {
            name: name.trim().to_lowercase(),
            alternate_names: vec![],
            option_type,
        }
    }

    fn add_alternate_name(mut self, name: &str) -> Self {
        self.alternate_names.push(name.trim().to_lowercase());
        self
    }

    fn new_spin<T: Clone + Copy + IntoSpin>(
        name: &str,
        values: SpinValue<T>,
        function: fn(&Engine, Spin),
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

    fn new_check(name: &str, default: bool, function: fn(&Engine, bool)) -> Self {
        UCIOption::new(name, UCIOptionType::Check { default, function })
    }

    fn new_button(name: &str, function: fn(&Engine)) -> Self {
        UCIOption::new(name, UCIOptionType::Button { function })
    }

    fn set_option(&self, engine: &Engine, value_string: String) -> Result<()> {
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
                format!("option name {} type check default {}", self.name, default)
            }
            UCIOptionType::Spin {
                default, min, max, ..
            } => {
                format!(
                    "option name {} type spin default {} min {} max {}",
                    self.name, default, min, max
                )
            }
            UCIOptionType::Combo {
                default, options, ..
            } => {
                format!(
                    "option name {} type combo default {} {}",
                    self.name,
                    default,
                    options.iter().map(|s| format!("var {s}")).join(" ")
                )
            }
            UCIOptionType::Button { .. } => {
                format!("option name {} type button", self.name)
            }
            UCIOptionType::String { default, .. } => {
                format!("option name {} type string default {}", self.name, default)
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
            options: RwLock::new(get_uci_options()),
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
                     name,
                     alternate_names,
                     ..
                 }| {
                    name.to_lowercase() == command_name || alternate_names.contains(&command_name)
                },
            )
            .cloned()
    }

    pub fn get_all_options(&self) -> Vec<UCIOption> {
        self.options.read().unwrap().to_owned()
    }

    pub fn run_command(&self, engine: &Engine, user_input: &str) -> Result<()> {
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

fn get_uci_options() -> Vec<UCIOption> {
    let default_uci_state = GlobalUCIState::default();
    let t_table_size_uci = SpinValue::new(
        default_uci_state.get_t_table_size(),
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
        default_uci_state.get_move_overhead(),
        Duration::from_secs(0),
        Duration::MAX,
    );

    let options = vec![
        UCIOption::new_spin(
            "Threads",
            SpinValue::new(default_uci_state.get_num_threads(), 1, 1024),
            |_, value| GLOBAL_UCI_STATE.set_num_threads(value as usize, true),
        )
        .add_alternate_name("Thread"),
        UCIOption::new_spin("Hash", t_table_size_uci, {
            |engine, value| {
                GLOBAL_UCI_STATE.set_t_table_size(
                    engine.get_transposition_table(),
                    CacheTableSize::Exact(value as usize),
                )
            }
        }),
        UCIOption::new_button("Clear Hash", |engine| {
            clear_all_cache_tables(engine.get_transposition_table())
        }),
        UCIOption::new_spin("Move Overhead", move_overhead_uci, |_, value| {
            GLOBAL_UCI_STATE.set_move_overhead(Duration::from_millis(value as u64))
        }),
        // UCIOption::new_check(
        //     "OwnBook",
        //     DEFAULT_USE_OWN_BOOK,
        //     GLOBAL_UCI_STATE.set_using_own_book,
        // ),
    ];
    options
}
