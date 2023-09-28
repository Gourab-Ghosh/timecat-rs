use super::*;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum UCIOptionType {
    Button {
        function: fn(),
    },
    Check {
        default: bool,
        function: fn(bool),
    },
    String {
        default: String,
        function: fn(&str),
    },
    Spin {
        default: Spin,
        min: Spin,
        max: Spin,
        function: fn(Spin),
    },
    Combo {
        default: String,
        options: Vec<String>,
        function: fn(&str),
    },
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct UCIOption {
    name: String,
    option_type: UCIOptionType,
}

impl UCIOption {
    fn new(name: &str, option_type: UCIOptionType) -> Self {
        Self {
            name: name.to_owned(),
            option_type,
        }
    }

    fn new_spin(name: &str, default: Spin, min: Spin, max: Spin, function: fn(Spin)) -> Self {
        UCIOption::new(
            name,
            UCIOptionType::Spin {
                default,
                min,
                max,
                function,
            },
        )
    }

    fn new_check(name: &str, default: bool, function: fn(bool)) -> Self {
        UCIOption::new(name, UCIOptionType::Check { default, function })
    }

    fn new_button(name: &str, function: fn()) -> Self {
        UCIOption::new(name, UCIOptionType::Button { function })
    }

    fn set_option(&self, value_string: String) -> Result<(), EngineError> {
        match &self.option_type {
            UCIOptionType::Check { function, .. } => {
                function(value_string.parse()?);
            }
            UCIOptionType::Spin {
                min, max, function, ..
            } => {
                let value = value_string.parse()?;
                if value < *min || value > *max {
                    return Err(EngineError::InvalidSpinValue {
                        name: self.name.to_owned(),
                        value,
                        min: *min,
                        max: *max,
                    });
                }
                function(value);
            }
            UCIOptionType::Combo { function, .. } => {
                function(&value_string);
            }
            UCIOptionType::Button { function } => {
                function();
            }
            UCIOptionType::String { function, .. } => {
                function(&value_string);
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

pub struct UCIOptionsMap {
    options: Mutex<Vec<UCIOption>>,
}

impl UCIOptionsMap {
    fn new() -> Self {
        Self {
            options: Mutex::new(get_uci_options()),
        }
    }

    pub fn get_option(&self, command_name: &str) -> Option<UCIOption> {
        self.options
            .lock()
            .unwrap()
            .iter()
            .find(|UCIOption { name, .. }| name.to_lowercase() == command_name)
            .cloned()
    }

    pub fn get_all_options(&self) -> Vec<UCIOption> {
        self.options.lock().unwrap().clone()
    }

    pub fn set_option(&self, command_name: &str, value_string: String) -> Result<(), EngineError> {
        self.get_option(command_name)
            .ok_or(EngineError::UnknownCommand)?
            .set_option(value_string)
    }
}

impl Default for UCIOptionsMap {
    fn default() -> Self {
        Self::new()
    }
}

fn get_uci_options() -> Vec<UCIOption> {
    let options = vec![
        UCIOption::new_spin(
            "Threads",
            DEFAULT_NUM_THREADS as Spin,
            MIN_NUM_THREADS as Spin,
            MAX_NUM_THREADS as Spin,
            |value| set_num_threads(value as usize, true),
        ),
        UCIOption::new_spin(
            "Hash",
            DEFAULT_T_TABLE_SIZE.unwrap() as Spin,
            MIN_T_TABLE_SIZE.unwrap() as Spin,
            MAX_T_TABLE_SIZE.unwrap() as Spin,
            |value| set_t_table_size(CacheTableSize::Max(value as usize)),
        ),
        UCIOption::new_button("Clear Hash", clear_all_hash_tables),
        UCIOption::new_spin(
            "Move Overhead",
            DEFAULT_MOVE_OVERHEAD.as_millis() as Spin,
            MIN_MOVE_OVERHEAD.as_millis() as Spin,
            MAX_MOVE_OVERHEAD.as_millis() as Spin,
            |value| set_move_overhead(Duration::from_millis(value as u64)),
        ),
        // UCIOption::new_check(
        //     "OwnBook",
        //     DEFAULT_USE_OWN_BOOK,
        //     set_using_own_book,
        // ),
    ];
    options
}
