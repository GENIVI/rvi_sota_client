use std::collections::HashMap;
use std::sync::{Mutex};
use std::sync::mpsc::{Sender, Receiver, channel};

use datatype::Event;

pub struct Registry {
    last_idx: Mutex<u32>,
    registry: HashMap<u32, Sender<Event>>,
    events_rx: Receiver<Event>
}

impl Registry {
    pub fn new(events_rx: Receiver<Event>) -> Registry {
        Registry { last_idx: Mutex::new(0), registry: HashMap::new(), events_rx: events_rx }
    }
    pub fn start(&self) {
        loop {
            let event = self.events_rx.recv().unwrap();
            for (_, subscriber) in &self.registry {
                let _ = subscriber.send(event.clone());
            }
        }
    }
    pub fn subscribe(&mut self) -> Receiver<Event> {
        let (tx, rx) = channel();
        let mut counter = self.last_idx.lock().unwrap();
        *counter += 1;
        self.registry.insert(*counter, tx);
        rx
    }
}
