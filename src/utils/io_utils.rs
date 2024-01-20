use super::*;
use std::io::{self, Write};

pub fn print_line<T: fmt::Display>(line: T) {
    let to_print = format!("{line}");
    if to_print.is_empty() {
        return;
    }
    print!("{to_print}");
    io::stdout().flush().unwrap();
}

pub struct IoReader {
    user_input: Mutex<String>,
    received_input: AtomicBool,
}

impl IoReader {
    pub fn new() -> Self {
        Self {
            user_input: Mutex::new(String::new()),
            received_input: AtomicBool::new(false),
        }
    }

    pub fn start_reader(&self) {
        loop {
            if self.received_input.load(MEMORY_ORDERING) {
                continue;
            }
            std::io::stdin()
                .read_line(&mut self.user_input.lock().unwrap())
                .expect("Failed to read line!");
            self.received_input.store(true, MEMORY_ORDERING);
        }
    }

    pub fn read_line_once(&self) -> Option<String> {
        if !self.received_input.load(MEMORY_ORDERING) {
            thread::sleep(Duration::from_millis(1));
            return None;
        }
        let mut user_input = self.user_input.lock().unwrap();
        let input = user_input.to_owned();
        user_input.clear();
        drop(user_input);
        self.received_input.store(false, MEMORY_ORDERING);
        Some(input)
    }

    pub fn read_line(&self) -> String {
        loop {
            if let Some(input) = self.read_line_once() {
                return input;
            }
        }
    }
}

impl Default for IoReader {
    fn default() -> Self {
        Self::new()
    }
}
