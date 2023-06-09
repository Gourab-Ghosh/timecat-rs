use super::*;
use std::io::{stdin, BufRead};

struct Communicator;

impl Communicator {
    pub fn new() -> Self {
        Self
    }

    pub fn communicate(&mut self) -> Option<String> {
        // let mut buf = String::new();
        // stdin().read_line(&mut buf).ok()?;
        // Some(buf)
        None
    }
}

pub struct Timer {
    start_instant: Instant,
    communication_check_instant: Option<Instant>,
    max_time: Option<Duration>,
    stop_search: bool,
    communicator: Communicator,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            start_instant: Instant::now(),
            communication_check_instant: None,
            max_time: None,
            stop_search: false,
            communicator: Communicator::new(),
        }
    }

    pub fn reset_variables(&mut self) {
        self.start_instant = Instant::now();
        self.communication_check_instant = None;
        self.max_time = None;
        self.stop_search = false;
        self.communicator = Communicator::new();
    }

    pub fn reset_start_time(&mut self) {
        self.start_instant = Instant::now();
        self.stop_search = false;
    }

    pub fn set_max_time(&mut self, duration: impl Into<Option<Duration>>) {
        self.max_time = duration.into();
        self.stop_search = false;
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

    pub fn start_communication_check(&mut self) {
        self.communication_check_instant = Some(Instant::now());
    }

    pub fn stop_communication_check(&mut self) {
        self.communication_check_instant = None;
    }

    pub fn check_communication(&mut self) -> bool {
        if let Some(instant) = self.communication_check_instant {
            if instant.elapsed() >= COMMUNICATION_CHECK_INTERVAL {
                if let Some(string) = self.communicator.communicate() {
                    if string.trim().to_lowercase() == "stop" {
                        self.stop_search = true;
                        self.stop_communication_check();
                        return true;
                    }
                }
                self.start_communication_check();
            }
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
        self.is_time_up() || self.check_communication()
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}
