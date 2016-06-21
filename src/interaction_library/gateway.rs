use chan::{Sender, Receiver};
use std::thread;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use super::broadcast::Broadcast;


#[derive(Clone, Debug)]
pub struct Interpret<C, E>
    where C: Send + Clone + Debug + 'static,
          E: Send + Clone + Debug + 'static,
{
    pub command:     C,
    pub response_tx: Option<Arc<Mutex<Sender<E>>>>,
}

pub trait Gateway<C, E>: Sized + Send + Sync + 'static
    where C: Send + Clone + Debug + 'static,
          E: Send + Clone + Debug + 'static,
{
    fn new(itx: Sender<Interpret<C, E>>, shutdown_rx: Receiver<()>) -> Self;

    fn run(itx: Sender<Interpret<C, E>>, erx: Receiver<E>, shutdown_rx: Receiver<()>) {
        let mut shutdown = Broadcast::new(shutdown_rx);
        let gateway      = Self::new(itx, shutdown.subscribe());

        let stop_pulse = shutdown.subscribe();
        thread::spawn(move || {
            loop {
                chan_select! {
                    stop_pulse.recv() => break,
                    erx.recv() -> e   => match e {
                        Some(e) => gateway.pulse(e),
                        None    => panic!("all gateway event transmitters are closed")
                    }
                }
            }
        });
    }

    fn pulse(&self, _: E) {} // ignore global events by default
}
