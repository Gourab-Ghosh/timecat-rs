use super::*;
use std::io::{stdin, BufRead};

#[derive(Clone, Debug)]
pub struct Timer {
    start_instant: Instant,
    max_time: Option<Duration>,
    stop_search: bool,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            start_instant: Instant::now(),
            max_time: None,
            stop_search: false,
        }
    }

    pub fn reset_variables(&mut self) {
        self.start_instant = Instant::now();
        self.max_time = None;
        self.stop_search = false;
    }

    pub fn reset_start_time(&mut self) {
        self.start_instant = Instant::now();
        self.stop_search = false;
    }

    pub fn set_max_time(&mut self, duration: impl Into<Option<Duration>>) {
        self.max_time = duration.into();
        self.stop_search = false;
    }

    pub fn get_clock(&self) -> Instant {
        self.start_instant
    }

    pub fn elapsed(&self) -> Duration {
        self.start_instant.elapsed()
    }

    pub fn max_time(&self) -> Option<Duration> {
        self.max_time
    }

    pub fn is_time_up(&mut self) -> bool {
        if let Some(max_time) = self.max_time {
            self.stop_search = self.elapsed() >= max_time;
            return self.stop_search;
        }
        false
    }

    pub fn check_stop(&mut self, enable_timer: bool) -> bool {
        if !enable_timer {
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
        Self::new()
    }
}
