use super::*;

#[cfg(feature = "engine")]
pub trait MeasureTime<T>: FnMut(&mut Engine) -> T {
    fn run_and_measure_time(&mut self, engine: &mut Engine) -> (T, Duration) {
        let clock = Instant::now();
        (self(engine), clock.elapsed())
    }

    fn run_and_print_time(&mut self, engine: &mut Engine) -> T {
        let (res, time_taken) = self.run_and_measure_time(engine);
        if GLOBAL_UCI_STATE.is_in_console_mode() {
            println!();
        }
        println_info("Run Time", time_taken.stringify());
        res
    }
}

#[cfg(feature = "engine")]
impl<T, Func: FnMut(&mut Engine) -> T> MeasureTime<T> for Func {}
