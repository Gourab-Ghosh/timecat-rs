use super::*;

static TERMINATE_ENGINE: AtomicBool = AtomicBool::new(false);
static COLORED_OUTPUT: AtomicBool = AtomicBool::new(true);
static CONSOLE_MODE: AtomicBool = AtomicBool::new(true);
static T_TABLE_SIZE: Mutex<CacheTableSize> = Mutex::new(T_TABLE_SIZE_UCI.get_default());
static LONG_ALGEBRAIC_NOTATION: AtomicBool = AtomicBool::new(false);
static NUM_THREADS: AtomicUsize = AtomicUsize::new(NUM_THREADS_UCI.get_default());
static MOVE_OVERHEAD: Mutex<Duration> = Mutex::new(MOVE_OVERHEAD_UCI.get_default());
static USE_OWN_BOOK: AtomicBool = AtomicBool::new(DEFAULT_USE_OWN_BOOK);
static DEBUG_MODE: AtomicBool = AtomicBool::new(DEFAULT_DEBUG_MODE);
static CHESS960_MODE: AtomicBool = AtomicBool::new(DEFAULT_CHESS960_MODE);

fn print_info<T: fmt::Display>(message: &str, info: impl Into<Option<T>>) {
    if !is_in_debug_mode() {
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
    if !is_in_console_mode() {
        to_print = format!("{} {to_print}", "info string".colorize(INFO_MESSAGE_STYLE))
    }
    println!("{to_print}");
}

#[inline(always)]
pub fn terminate_engine() -> bool {
    TERMINATE_ENGINE.load(MEMORY_ORDERING)
}

pub fn set_engine_termination(b: bool) {
    TERMINATE_ENGINE.store(b, MEMORY_ORDERING);
}

#[inline(always)]
pub fn is_colored_output() -> bool {
    COLORED_OUTPUT.load(MEMORY_ORDERING)
}

pub fn set_colored_output(b: bool, print: bool) {
    COLORED_OUTPUT.store(b, MEMORY_ORDERING);
    if print {
        print_info("Colored output is set to", b);
    }
}

#[inline(always)]
pub fn is_in_console_mode() -> bool {
    CONSOLE_MODE.load(MEMORY_ORDERING)
}

pub fn set_console_mode(b: bool, print: bool) {
    CONSOLE_MODE.store(b, MEMORY_ORDERING);
    DEBUG_MODE.store(b, MEMORY_ORDERING);
    if print {
        print_info("UCI mode is set to", b);
    }
}

#[inline(always)]
pub fn get_t_table_size() -> CacheTableSize {
    T_TABLE_SIZE.lock().unwrap().to_owned()
}

pub fn set_t_table_size(size: CacheTableSize) {
    *T_TABLE_SIZE.lock().unwrap() = size;
    TRANSPOSITION_TABLE.reset_size();
    if is_in_debug_mode() {
        TRANSPOSITION_TABLE.print_info();
    }
    print_info(
        "Transposition table is set to size to",
        size.to_cache_table_memory_size::<TranspositionTableEntry>(),
    );
}

#[inline(always)]
pub fn use_long_algebraic_notation() -> bool {
    LONG_ALGEBRAIC_NOTATION.load(MEMORY_ORDERING)
}

pub fn set_long_algebraic_notation(b: bool) {
    LONG_ALGEBRAIC_NOTATION.store(b, MEMORY_ORDERING);
    print_info("Long algebraic notation is set to", b);
}

#[inline(always)]
pub fn get_num_threads() -> usize {
    NUM_THREADS.load(MEMORY_ORDERING)
}

pub fn set_num_threads(num_threads: usize, print: bool) {
    NUM_THREADS.store(num_threads, MEMORY_ORDERING);
    if print {
        print_info("Number of threads is set to", num_threads);
    }
}

#[inline(always)]
pub fn get_move_overhead() -> Duration {
    MOVE_OVERHEAD.lock().unwrap().to_owned()
}

pub fn set_move_overhead(duration: Duration) {
    *MOVE_OVERHEAD.lock().unwrap() = duration;
    print_info("Move Overhead is set to", duration.stringify());
}

#[inline(always)]
pub fn use_own_book() -> bool {
    USE_OWN_BOOK.load(MEMORY_ORDERING)
}

pub fn set_using_own_book(b: bool) {
    USE_OWN_BOOK.store(b, MEMORY_ORDERING);
    print_info("Own Book Usage is set to", b);
}

#[inline(always)]
pub fn is_in_debug_mode() -> bool {
    DEBUG_MODE.load(MEMORY_ORDERING)
}

pub fn set_debug_mode(b: bool) {
    DEBUG_MODE.store(b, MEMORY_ORDERING);
    print_info("Debug Mode is set to", b);
}

#[inline(always)]
pub fn is_in_console_and_debug_mode() -> bool {
    is_in_console_mode() && is_in_debug_mode()
}

pub fn clear_all_hash_tables() {
    TRANSPOSITION_TABLE.clear();
    EVALUATOR.clear();
    print_info::<&str>("All hash tables are cleared!", None);
}

#[inline(always)]
pub fn is_in_chess960_mode() -> bool {
    CHESS960_MODE.load(MEMORY_ORDERING)
}

pub fn set_chess960_mode(b: bool) {
    CHESS960_MODE.store(b, MEMORY_ORDERING);
    print_info("Debug Mode is set to", b);
}
