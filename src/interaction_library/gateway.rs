use chan::{Sender, Receiver};
use std;
use std::thread;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};


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
    fn new(itx: Sender<Interpret<C, E>>) -> Result<Self, String>;

    fn run(itx: Sender<Interpret<C, E>>, erx: Receiver<E>) {
        let gateway = Self::new(itx).unwrap_or_else(|err| {
            error!("couldn't start gateway: {}", err);
            std::process::exit(1);
        });

        thread::spawn(move || {
            loop {
                gateway.pulse(erx.recv().expect("all gateway event transmitters are closed"));
            }
        });
    }

    fn pulse(&self, _: E) {} // ignore global events by default
}
