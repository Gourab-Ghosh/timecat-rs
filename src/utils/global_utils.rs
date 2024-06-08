use super::*;

pub fn identity_function<T>(object: T) -> T {
    object
}

fn print_info<T: fmt::Display>(message: &str, info: impl Into<Option<T>>) {
    if !GLOBAL_UCI_STATE.is_in_debug_mode() {
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
    if GLOBAL_UCI_STATE.is_in_uci_mode() {
        to_print = format!("{} {to_print}", "info string".colorize(INFO_MESSAGE_STYLE))
    }
    println!("{to_print}");
}

#[derive(Debug)]
pub struct GlobalUCIState {
    _terminate_engine: AtomicBool,
    #[cfg(feature = "colored_output")]
    _colored_output: AtomicBool,
    _console_mode: AtomicBool,
    _t_table_size: Mutex<CacheTableSize>,
    _long_algebraic_notation: AtomicBool,
    _num_threads: AtomicUsize,
    _move_overhead: Mutex<Duration>,
    _use_own_book: AtomicBool,
    _debug_mode: AtomicBool,
    _chess960_mode: AtomicBool,
}

impl Default for GlobalUCIState {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalUCIState {
    pub const fn new() -> Self {
        GlobalUCIState {
            _terminate_engine: AtomicBool::new(false),
            #[cfg(feature = "colored_output")]
            _colored_output: AtomicBool::new(true),
            _console_mode: AtomicBool::new(true),
            _t_table_size: Mutex::new(CacheTableSize::Exact(16)),
            _long_algebraic_notation: AtomicBool::new(false),
            _num_threads: AtomicUsize::new(1),
            _move_overhead: Mutex::new(Duration::ZERO),
            _use_own_book: AtomicBool::new(false),
            _debug_mode: AtomicBool::new(true),
            _chess960_mode: AtomicBool::new(false),
        }
    }

    #[inline(always)]
    pub fn terminate_engine(&self) -> bool {
        self._terminate_engine.load(MEMORY_ORDERING)
    }

    #[inline(always)]
    pub fn set_engine_termination(&self, b: bool) {
        self._terminate_engine.store(b, MEMORY_ORDERING);
    }

    #[cfg(feature = "colored_output")]
    #[inline(always)]
    pub fn is_colored_output(&self) -> bool {
        self._colored_output.load(MEMORY_ORDERING)
    }

    #[cfg(not(feature = "colored_output"))]
    #[inline(always)]
    pub fn is_colored_output(&self) -> bool {
        false
    }

    #[cfg(feature = "colored_output")]
    pub fn set_colored_output(&self, b: bool, print: bool) {
        self._colored_output.store(b, MEMORY_ORDERING);
        if print {
            print_info("Colored output is set to", b);
        }
    }

    #[inline(always)]
    pub fn is_in_console_mode(&self) -> bool {
        self._console_mode.load(MEMORY_ORDERING)
    }

    #[inline(always)]
    pub fn is_in_uci_mode(&self) -> bool {
        !self.is_in_console_mode()
    }

    pub fn set_console_mode(&self, b: bool, print: bool) {
        self._console_mode.store(b, MEMORY_ORDERING);
        self._debug_mode.store(b, MEMORY_ORDERING);
        if print {
            print_info("UCI mode is set to", b);
        }
    }

    #[inline(always)]
    pub fn set_uci_mode(&self, b: bool, print: bool) {
        self.set_console_mode(!b, print);
    }

    #[inline(always)]
    pub fn set_to_uci_mode(&self) {
        self.set_uci_mode(true, false);
    }

    #[inline(always)]
    pub fn set_to_console_mode(&self) {
        self.set_console_mode(true, false);
    }

    #[cfg(feature = "engine")]
    #[inline(always)]
    pub fn get_t_table_size(&self) -> CacheTableSize {
        self._t_table_size.lock().unwrap().to_owned()
    }

    #[cfg(feature = "engine")]
    pub fn set_t_table_size(&self, transposition_table: &TranspositionTable, size: CacheTableSize) {
        //TODO: modify such that T Table and evaluation function takes same amount of space
        *self._t_table_size.lock().unwrap() = size;
        transposition_table.reset_size();
        if GLOBAL_UCI_STATE.is_in_debug_mode() {
            transposition_table.print_info();
        }
        print_info(
            "Transposition table is set to size to",
            size.to_cache_table_memory_size::<TranspositionTableEntry>(),
        );
    }

    #[inline(always)]
    pub fn use_long_algebraic_notation(&self) -> bool {
        self._long_algebraic_notation.load(MEMORY_ORDERING)
    }

    pub fn set_long_algebraic_notation(&self, b: bool) {
        self._long_algebraic_notation.store(b, MEMORY_ORDERING);
        print_info("Long algebraic notation is set to", b);
    }

    #[inline(always)]
    pub fn get_num_threads(&self) -> usize {
        self._num_threads.load(MEMORY_ORDERING)
    }

    pub fn set_num_threads(&self, num_threads: usize, print: bool) {
        self._num_threads.store(num_threads, MEMORY_ORDERING);
        if print {
            print_info("Number of threads is set to", num_threads);
        }
    }

    #[inline(always)]
    pub fn get_move_overhead(&self) -> Duration {
        self._move_overhead.lock().unwrap().to_owned()
    }

    pub fn set_move_overhead(&self, duration: Duration) {
        *self._move_overhead.lock().unwrap() = duration;
        print_info("Move Overhead is set to", duration.stringify());
    }

    #[inline(always)]
    pub fn use_own_book(&self) -> bool {
        self._use_own_book.load(MEMORY_ORDERING)
    }

    pub fn set_using_own_book(&self, b: bool) {
        self._use_own_book.store(b, MEMORY_ORDERING);
        print_info("Own Book Usage is set to", b);
    }

    #[inline(always)]
    pub fn is_in_debug_mode(&self) -> bool {
        self._debug_mode.load(MEMORY_ORDERING)
    }

    pub fn set_debug_mode(&self, b: bool) {
        self._debug_mode.store(b, MEMORY_ORDERING);
        print_info("Debug Mode is set to", b);
    }

    #[inline(always)]
    pub fn is_in_console_and_debug_mode(&self) -> bool {
        self.is_in_console_mode() && self.is_in_debug_mode()
    }

    #[inline(always)]
    pub fn is_in_chess960_mode(&self) -> bool {
        self._chess960_mode.load(MEMORY_ORDERING)
    }

    pub fn set_chess960_mode(&self, b: bool) {
        self._chess960_mode.store(b, MEMORY_ORDERING);
        print_info("Chess 960 mode is set to", b);
    }
}

#[cfg(feature = "engine")]
pub fn clear_all_cache_tables(transposition_table: &TranspositionTable) {
    transposition_table.clear();
    #[cfg(feature = "nnue")]
    EVALUATOR.clear();
    print_info::<&str>("All hash tables are cleared!", None);
}
