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

use super::*;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct CustomDebug<T> {
    item: T,
    debug_message: Arc<String>,
}

impl<T> CustomDebug<T> {
    pub fn new(item: T, debug_message: &str) -> Self {
        Self {
            item,
            debug_message: Arc::new(debug_message.to_string()),
        }
    }

    pub fn get_debug_message(&self) -> &str {
        &self.debug_message
    }
}

impl<T: Debug> Debug for CustomDebug<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.debug_message.is_empty() {
            write!(f, "{:?}", self.item)
        } else {
            write!(f, "{}", self.debug_message)
        }
    }
}

impl<T> Deref for CustomDebug<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<T> DerefMut for CustomDebug<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.item
    }
}

impl<T> From<T> for CustomDebug<T> {
    fn from(value: T) -> Self {
        Self::new(value, "")
    }
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
