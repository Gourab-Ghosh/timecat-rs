use super::*;

pub trait MeasureTime<T>: Fn() -> T {
    fn run_and_measure_time(&self) -> (T, Duration) {
        let clock = Instant::now();
        (self(), clock.elapsed())
    }

    fn run_and_print_time(&self) -> T {
        let (res, time_taken) = self.run_and_measure_time();
        if UCI_STATE.is_in_console_mode() {
            println!();
        }
        println_info("Run Time", time_taken.stringify());
        res
    }
}

impl<T, Func: Fn() -> T> MeasureTime<T> for Func {}
