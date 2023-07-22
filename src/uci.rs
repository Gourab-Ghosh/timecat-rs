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

    fn set_option(&self, value: Option<String>) -> Result<(), EngineError> {
        match &self.option_type {
            UCIOptionType::Check { function, .. } => {
                let value = value.unwrap().parse()?;
                function(value);
            }
            UCIOptionType::Spin {
                min, max, function, ..
            } => {
                let value = value.unwrap().parse()?;
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
                function(&value.unwrap());
            }
            UCIOptionType::Button { function } => {
                function();
            }
            UCIOptionType::String { function, .. } => {
                function(&value.unwrap());
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
    options: Mutex<HashMap<String, UCIOptionType>>,
}

impl UCIOptionsMap {
    fn new() -> Self {
        let uci_map = Self {
            options: Default::default(),
        };
        {
            let mut inner_map = uci_map.options.lock().unwrap();
            for UCIOption { name, option_type } in get_uci_options() {
                inner_map.insert(name, option_type);
            }
        }
        uci_map
    }

    pub fn get_option(&self, name: &str) -> Option<UCIOption> {
        self.options
            .lock()
            .unwrap()
            .get(name)
            .map(|option_type| UCIOption::new(name, option_type.clone()))
    }

    pub fn get_all_options(&self) -> Vec<UCIOption> {
        let mut all_options = self
            .options
            .lock()
            .unwrap()
            .iter()
            .map(|(name, option_type)| UCIOption::new(name, option_type.clone()))
            .collect_vec();
        all_options.sort_unstable();
        all_options
    }

    pub fn set_option(
        &self,
        name: &str,
        value_string: impl Into<Option<String>>,
    ) -> Result<(), EngineError> {
        if let Some(option) = self.get_option(name) {
            option.set_option(value_string.into())
        } else {
            Err(EngineError::UnknownCommand)
        }
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
