use super::*;

pub fn format_info<T: fmt::Display>(desc: &str, info: T) -> String {
    let mut desc = desc.trim().trim_end_matches(':').to_string();
    if !is_in_console_mode() {
        desc = desc.to_lowercase();
    }
    desc = desc.colorize(INFO_MESSAGE_STYLE);
    if is_in_console_mode() {
        format!("{desc}: {info}")
    } else {
        format!(
            "{} {desc} {info}",
            "info string".colorize(INFO_MESSAGE_STYLE),
        )
    }
}

pub fn force_println_info<T: fmt::Display>(desc: &str, info: T) {
    println!("{}", format_info(desc, info));
}

#[inline(always)]
pub fn println_info<T: fmt::Display>(desc: &str, info: T) {
    if is_in_debug_mode() {
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

pub fn print_engine_info() {
    print_engine_version(true);
    println!();
    TRANSPOSITION_TABLE.print_info();
    EVALUATOR.print_info();
}

pub fn print_cache_table_info(
    name: &str,
    table_len: impl fmt::Display,
    table_size: impl fmt::Display,
) {
    let mut to_print = format!(
        "{name} initialization complete with {table_len} entries taking {table_size} space."
    );
    if !is_in_console_mode() {
        to_print = "info string ".to_string() + to_print.trim();
    }
    println!("{}", to_print.colorize(INFO_MESSAGE_STYLE));
}
