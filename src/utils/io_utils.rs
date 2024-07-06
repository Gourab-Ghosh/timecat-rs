use super::*;
use std::io::{self, Write};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;

pub fn print_line<T: fmt::Display>(line: T) {
    let to_print = format!("{line}");
    if to_print.is_empty() {
        return;
    }
    print_wasm!("{to_print}");
    io::stdout().flush().unwrap();
}

pub fn get_input<T: fmt::Display>(q: T, io_reader: &IoReader) -> String {
    print_line(q);
    io_reader.read_line()
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CustomDebug<T> {
    item: T,
    debug_message_func: Arc<fn(&T) -> String>,
}

impl<T> CustomDebug<T> {
    pub fn new(item: T, debug_message_func: fn(&T) -> String) -> Self {
        Self {
            item,
            debug_message_func: Arc::new(debug_message_func),
        }
    }

    pub fn get_debug_message_func(&self) -> &fn(&T) -> String {
        &self.debug_message_func
    }

    pub fn get_debug_message(&self) -> String {
        (self.debug_message_func)(&self.item)
    }

    pub fn into_inner(&self) -> &T {
        &self.item
    }

    pub fn into_inner_mut(&mut self) -> &mut T {
        &mut self.item
    }
}

impl<T: Debug> Debug for CustomDebug<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let debug_message = self.get_debug_message();
        write!(f, "{}", debug_message)
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

impl<T: Debug> From<T> for CustomDebug<T> {
    fn from(value: T) -> Self {
        Self::new(value, |item| format!("{item:?}"))
    }
}

#[derive(Clone, Debug)]
pub struct IoReader {
    sender: Sender<String>,
    receiver: Arc<Mutex<Receiver<String>>>,
}

impl IoReader {
    pub fn new() -> Self {
        let (sender, receiver) = channel();
        Self {
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }

    pub fn start_reader_in_parallel(&self) -> thread::JoinHandle<()> {
        let sender = self.sender.clone();
        thread::spawn(move || loop {
            let mut user_input = String::new();
            std::io::stdin()
                .read_line(&mut user_input)
                .expect("Failed to read line!");
            sender.send(user_input).unwrap();
        })
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
