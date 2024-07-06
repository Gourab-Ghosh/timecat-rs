use super::*;

#[cfg(feature = "engine")]
pub trait MeasureTime0<T> {
    fn run_and_measure_time(&mut self) -> (T, Duration);
    fn run_and_print_time(&mut self) -> T;
}

#[cfg(feature = "engine")]
impl<T, Func: FnMut() -> T> MeasureTime0<T> for Func {
    fn run_and_measure_time(&mut self) -> (T, Duration) {
        let clock = Instant::now();
        (self(), clock.elapsed())
    }

    fn run_and_print_time(&mut self) -> T {
        let (res, time_taken) = self.run_and_measure_time();
        if GLOBAL_TIMECAT_STATE.is_in_console_mode() {
            println_wasm!();
        }
        println_info("Run Time", time_taken.stringify());
        res
    }
}

#[cfg(feature = "engine")]
pub trait MeasureTime1<T, U> {
    fn run_and_measure_time(&mut self, item: &mut U) -> (T, Duration);
    fn run_and_print_time(&mut self, item: &mut U) -> T;
}

#[cfg(feature = "engine")]
impl<T, Func: FnMut(&mut U) -> T, U> MeasureTime1<T, U> for Func {
    fn run_and_measure_time(&mut self, item: &mut U) -> (T, Duration) {
        let clock = Instant::now();
        (self(item), clock.elapsed())
    }

    fn run_and_print_time(&mut self, item: &mut U) -> T {
        let (res, time_taken) = self.run_and_measure_time(item);
        if GLOBAL_TIMECAT_STATE.is_in_console_mode() {
            println_wasm!();
        }
        println_info("Run Time", time_taken.stringify());
        res
    }
}
