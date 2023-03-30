use super::*;

pub struct Timer {
    start_time: Instant,
    max_time: Option<Duration>,
    stop_search: bool,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            max_time: None,
            stop_search: false,
        }
    }

    pub fn reset_variables(&mut self) {
        self.start_time = Instant::now();
        self.max_time = None;
        self.stop_search = false;
    }

    pub fn reset_start_time(&mut self) {
        self.start_time = Instant::now();
        self.stop_search = false;
    }

    pub fn set_max_time(&mut self, duration: Option<Duration>) {
        self.max_time = duration;
        self.stop_search = false;
    }

    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    pub fn max_time(&self) -> Option<Duration> {
        self.max_time
    }

    pub fn stop_search(&self) -> bool {
        self.stop_search
    }

    pub fn is_time_up(&mut self) -> bool {
        if let Some(max_time) = self.max_time {
            self.stop_search = self.elapsed() >= max_time;
            return self.stop_search;
        }
        false
    }
}
