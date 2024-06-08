use super::*;

#[cfg(feature = "engine")]
pub fn format_info<T: fmt::Display>(desc: &str, info: T, add_info_string: bool) -> String {
    let mut desc = desc.trim().trim_end_matches(':').to_string();
    if UCI_STATE.is_in_uci_mode() {
        desc = desc.to_lowercase();
    }
    desc = desc.colorize(INFO_MESSAGE_STYLE);
    if UCI_STATE.is_in_console_mode() {
        format!("{desc}: {info}")
    } else {
        let mut formatted_info = format!("{desc} {info}",);
        if add_info_string {
            formatted_info = "info string".colorize(INFO_MESSAGE_STYLE) + &formatted_info
        }
        formatted_info
    }
}

#[cfg(not(feature = "engine"))]
pub fn format_info<T: fmt::Display>(desc: &str, info: T, _: bool) -> String {
    let desc = desc.trim().trim_end_matches(':').colorize(INFO_MESSAGE_STYLE);
    format!("{desc}: {info}")
}

#[cfg(feature = "engine")]
pub fn force_println_info<T: fmt::Display>(desc: &str, info: T) {
    println!("{}", format_info(desc, info, true));
}

#[cfg(feature = "engine")]
#[inline(always)]
pub fn println_info<T: fmt::Display>(desc: &str, info: T) {
    if UCI_STATE.is_in_debug_mode() {
        force_println_info(desc, info);
    }
}

#[inline(always)]
pub fn get_engine_version() -> String {
    format!("{ENGINE_NAME} v{ENGINE_VERSION}")
}

pub fn print_engine_version(color: bool) {
    let version = get_engine_version();
    if color {
        println!("{}", version.colorize(SUCCESS_MESSAGE_STYLE));
        return;
    }
    println!("{version}");
}

#[cfg(feature = "engine")]
pub fn print_engine_info(transposition_table: &TranspositionTable) {
    print_engine_version(true);
    println!();
    transposition_table.print_info();
    #[cfg(feature = "nnue")]
    EVALUATOR.print_info();
}

#[cfg(feature = "engine")]
pub fn print_cache_table_info(
    name: &str,
    table_len: impl fmt::Display,
    table_size: impl fmt::Display,
) {
    let mut to_print = format!(
        "{name} initialization complete with {table_len} entries taking {table_size} space."
    );
    if UCI_STATE.is_in_uci_mode() {
        to_print = "info string ".to_string() + to_print.trim();
    }
    println!("{}", to_print.colorize(INFO_MESSAGE_STYLE));
}
