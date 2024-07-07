use super::*;

pub fn identity_function<T>(object: T) -> T {
    object
}

pub fn print_uci_info<T: fmt::Display>(message: &str, info: impl Into<Option<T>>) {
    if !GLOBAL_TIMECAT_STATE.is_in_debug_mode() {
        return;
    }
    let mut to_print = if let Some(info_message) = info.into() {
        format!(
            "{} {}",
            message.colorize(SUCCESS_MESSAGE_STYLE),
            info_message.colorize(INFO_MESSAGE_STYLE),
        )
    } else {
        message.colorize(SUCCESS_MESSAGE_STYLE)
    };
    if GLOBAL_TIMECAT_STATE.is_in_uci_mode() {
        to_print = format!("{} {to_print}", "info string".colorize(INFO_MESSAGE_STYLE))
    }
    println_wasm!("{to_print}");
}

pub struct TimecatDefaults {
    #[cfg(feature = "colored")]
    pub colored: bool,
    pub console_mode: bool,
    pub t_table_size: CacheTableSize,
    pub long_algebraic_notation: bool,
    pub num_threads: NonZeroUsize,
    pub move_overhead: Duration,
    pub use_own_book: bool,
    pub debug_mode: bool,
    pub chess960_mode: bool,
}

#[derive(Debug)]
pub struct GlobalTimecatState {
    #[cfg(feature = "colored")]
    _colored: AtomicBool,
    _console_mode: AtomicBool,
    _long_algebraic_notation: AtomicBool,
    _debug_mode: AtomicBool,
}

impl Default for GlobalTimecatState {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalTimecatState {
    pub const fn new() -> Self {
        GlobalTimecatState {
            #[cfg(feature = "colored")]
            _colored: AtomicBool::new(TIMECAT_DEFAULTS.colored),
            _console_mode: AtomicBool::new(TIMECAT_DEFAULTS.console_mode),
            _long_algebraic_notation: AtomicBool::new(TIMECAT_DEFAULTS.long_algebraic_notation),
            _debug_mode: AtomicBool::new(TIMECAT_DEFAULTS.debug_mode),
        }
    }

    #[cfg(feature = "colored")]
    #[inline]
    pub fn is_colored(&self) -> bool {
        self._colored.load(MEMORY_ORDERING)
    }

    #[cfg(not(feature = "colored"))]
    #[inline]
    pub fn is_colored(&self) -> bool {
        false
    }

    #[cfg(feature = "colored")]
    pub fn set_colored(&self, b: bool, verbose: bool) {
        self._colored.store(b, MEMORY_ORDERING);
        if verbose {
            print_uci_info("Colored output is set to", b);
        }
    }

    #[inline]
    pub fn is_in_console_mode(&self) -> bool {
        self._console_mode.load(MEMORY_ORDERING)
    }

    #[inline]
    pub fn is_in_uci_mode(&self) -> bool {
        !self.is_in_console_mode()
    }

    pub fn set_console_mode(&self, b: bool, verbose: bool) {
        self._console_mode.store(b, MEMORY_ORDERING);
        self._debug_mode.store(b, MEMORY_ORDERING);
        if verbose {
            force_println_info("Console mode is set to", b);
        }
    }

    pub fn set_uci_mode(&self, b: bool, verbose: bool) {
        self.set_console_mode(!b, false);
        if verbose {
            force_println_info("UCI mode is set to", b);
        }
    }

    #[inline]
    pub fn set_to_uci_mode(&self) {
        self.set_uci_mode(true, false);
    }

    #[inline]
    pub fn set_to_console_mode(&self) {
        self.set_console_mode(true, false);
    }

    #[inline]
    pub fn use_long_algebraic_notation(&self) -> bool {
        self._long_algebraic_notation.load(MEMORY_ORDERING)
    }

    pub fn set_long_algebraic_notation(&self, b: bool) {
        self._long_algebraic_notation.store(b, MEMORY_ORDERING);
        print_uci_info("Long algebraic notation is set to", b);
    }

    #[inline]
    pub fn is_in_debug_mode(&self) -> bool {
        self._debug_mode.load(MEMORY_ORDERING)
    }

    pub fn set_debug_mode(&self, b: bool) {
        self._debug_mode.store(b, MEMORY_ORDERING);
        print_uci_info("Debug Mode is set to", b);
    }
}
