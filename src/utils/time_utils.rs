use super::*;

pub fn measure_time<T>(func: impl Fn() -> T) -> T {
    let clock = Instant::now();
    let res = func();
    if is_in_console_mode() {
        println!();
    }
    println_info("Run Time", clock.elapsed().stringify());
    res
}
