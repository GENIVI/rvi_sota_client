use chan::{Sender, Receiver};
use std::process;
use std::sync::{Arc, Mutex};

use datatype::{Command, Event};


pub struct Interpret {
    pub command:     Command,
    pub response_tx: Option<Arc<Mutex<Sender<Event>>>>,
}

pub trait Gateway {
    fn initialize(&mut self, itx: Sender<Interpret>) -> Result<(), String>;

    fn start(&mut self, itx: Sender<Interpret>, erx: Receiver<Event>) {
        self.initialize(itx).unwrap_or_else(|err| {
            error!("couldn't start gateway: {}", err);
            process::exit(1);
        });

        loop {
            self.pulse(erx.recv().expect("all gateway event transmitters are closed"));
        }
    }

    fn pulse(&self, _: Event) {} // ignore global events by default
}
