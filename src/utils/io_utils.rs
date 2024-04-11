use super::*;
use std::io::{self, Write};
use std::sync::mpsc::{channel, Receiver, Sender};

pub fn print_line<T: fmt::Display>(line: T) {
    let to_print = format!("{line}");
    if to_print.is_empty() {
        return;
    }
    print!("{to_print}");
    io::stdout().flush().unwrap();
}

pub struct IoReader {
    sender: Sender<String>,
    receiver: Mutex<Receiver<String>>,
}

impl IoReader {
    pub fn new() -> Self {
        let (sender, receiver) = channel();
        Self {
            sender,
            receiver: Mutex::new(receiver),
        }
    }

    pub fn start_reader(&self) {
        loop {
            let mut user_input = String::new();
            std::io::stdin()
                .read_line(&mut user_input)
                .expect("Failed to read line!");
            self.sender.send(user_input).unwrap();
        }
    }

    pub fn read_line_once(&self) -> Option<String> {
        self.receiver
            .lock()
            .unwrap()
            .recv_timeout(COMMUNICATION_CHECK_INTERVAL)
            .ok()
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
