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
        if GLOBAL_UCI_STATE.is_in_console_mode() {
            println!();
        }
        println_info("Run Time", time_taken.stringify());
        res
    }
}

#[cfg(feature = "engine")]
pub trait MeasureTime1<T> {
    fn run_and_measure_time(&mut self, engine: &mut Engine) -> (T, Duration);
    fn run_and_print_time(&mut self, engine: &mut Engine) -> T;
}

#[cfg(feature = "engine")]
impl<T, Func: FnMut(&mut Engine) -> T> MeasureTime1<T> for Func {
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
pub trait MeasureTime2<T> {
    fn run_and_measure_time(&mut self, engine: &mut Engine, io_reader: &IoReader) -> (T, Duration);
    fn run_and_print_time(&mut self, engine: &mut Engine, io_reader: &IoReader) -> T;
}

#[cfg(feature = "engine")]
impl<T, Func: FnMut(&mut Engine, &IoReader) -> T> MeasureTime2<T> for Func {
    fn run_and_measure_time(&mut self, engine: &mut Engine, io_reader: &IoReader) -> (T, Duration) {
        let clock = Instant::now();
        (self(engine, io_reader), clock.elapsed())
    }

    fn run_and_print_time(&mut self, engine: &mut Engine, io_reader: &IoReader) -> T {
        let (res, time_taken) = self.run_and_measure_time(engine, io_reader);
        if GLOBAL_UCI_STATE.is_in_console_mode() {
            println!();
        }
        println_info("Run Time", time_taken.stringify());
        res
    }
}

#[cfg(feature = "engine")]
pub trait MeasureTime3<T> {
    fn run_and_measure_time(
        &mut self,
        engine: &mut Engine,
        uci_options: &UCIStateManager,
        io_reader: &IoReader,
    ) -> (T, Duration);
    fn run_and_print_time(
        &mut self,
        engine: &mut Engine,
        uci_options: &UCIStateManager,
        io_reader: &IoReader,
    ) -> T;
}

#[cfg(feature = "engine")]
impl<T, Func: FnMut(&mut Engine, &UCIStateManager, &IoReader) -> T> MeasureTime3<T> for Func {
    fn run_and_measure_time(
        &mut self,
        engine: &mut Engine,
        uci_options: &UCIStateManager,
        io_reader: &IoReader,
    ) -> (T, Duration) {
        let clock = Instant::now();
        (self(engine, uci_options, io_reader), clock.elapsed())
    }

    fn run_and_print_time(
        &mut self,
        engine: &mut Engine,
        uci_options: &UCIStateManager,
        io_reader: &IoReader,
    ) -> T {
        let (res, time_taken) = self.run_and_measure_time(engine, uci_options, io_reader);
        if GLOBAL_UCI_STATE.is_in_console_mode() {
            println!();
        }
        println_info("Run Time", time_taken.stringify());
        res
    }
}
