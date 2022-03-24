use std::io::{stdout, Write};
use crossbeam::channel::{Receiver};

pub struct Printer {
    print_recv: Receiver<String>
}

impl Printer {
    pub fn new(print_recv: Receiver<String>) -> Self {
        Self {
            print_recv
        }
    }

    pub fn run(&self) {
        loop {
            let message = self.print_recv.recv().unwrap();
            if message.is_empty() {
                return
            }
            writeln!(stdout(), "{}", message).unwrap();
        }
    }
}