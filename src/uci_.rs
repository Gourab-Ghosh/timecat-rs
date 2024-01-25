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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SpinValue<T: Clone + Copy + IntoSpin> {
    default: T,
    min: T,
    max: T,
}

impl<T: Clone + Copy + IntoSpin> SpinValue<T> {
    #[inline(always)]
    pub const fn new(default: T, min: T, max: T) -> Self {
        Self { default, min, max }
    }

    #[inline(always)]
    pub const fn get_default(self) -> T {
        self.default
    }

    #[inline(always)]
    pub const fn get_min(self) -> T {
        self.min
    }

    #[inline(always)]
    pub const fn get_max(self) -> T {
        self.max
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UCIOption {
    Thread(SpinValue<usize>),
    Hash(SpinValue<CacheTableSize>)
}

pub struct UCIOptions {
    options: Vec<UCIOption>
}