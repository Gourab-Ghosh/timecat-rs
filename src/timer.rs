use super::*;

#[derive(Clone, Debug)]
pub struct Timer {
    start_instant: Instant,
    max_time: Duration,
    stop_search: bool,
    stopper: Arc<AtomicBool>,
    is_dummy: bool,
}

impl Timer {
    pub fn new(stopper: Arc<AtomicBool>) -> Self {
        Self {
            start_instant: Instant::now(),
            max_time: Duration::MAX,
            stop_search: false,
            stopper,
            is_dummy: false,
        }
    }

    pub fn new_dummy(stopper: Arc<AtomicBool>) -> Self {
        timer::Timer {
            is_dummy: true,
            stopper,
            ..Default::default()
        }
    }

    pub fn reset_variables(&mut self) {
        self.start_instant = Instant::now();
        self.max_time = Duration::MAX;
        self.stop_search = false;
    }

    pub fn reset_start_time(&mut self) {
        self.start_instant = Instant::now();
        self.stop_search = false;
    }

    pub fn set_max_time(&mut self, duration: Duration) {
        self.max_time = duration;
        self.stop_search = false;
    }

    pub fn update_max_time(&mut self, duration: Duration) {
        if self.max_time != Duration::MAX {
            self.set_max_time(self.max_time.min(duration));
        }
    }

    pub fn get_clock(&self) -> Instant {
        self.start_instant
    }

    pub fn time_elapsed(&self) -> Duration {
        self.start_instant.elapsed()
    }

    pub fn max_time(&self) -> Duration {
        self.max_time
    }

    pub fn is_time_up(&mut self) -> bool {
        if self.max_time == Duration::MAX {
            return false;
        }
        self.stop_search = self.time_elapsed() + UCI_STATE.get_move_overhead() >= self.max_time;
        self.stop_search
    }

    pub fn check_stop(&mut self, enable_timer: bool) -> bool {
        if self.stopper.load(MEMORY_ORDERING) {
            return true;
        }
        if self.is_dummy || !enable_timer {
            return false;
        }
        if self.stop_search {
            return true;
        }
        self.is_time_up()
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new(Arc::new(AtomicBool::new(false)))
    }
}
