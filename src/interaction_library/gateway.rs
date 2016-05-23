use std::thread;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Sender, Receiver};


pub struct Interpret<C, E> {
    pub cmd: C,
    pub etx: Option<Arc<Mutex<Sender<E>>>>,
}

pub trait Gateway<C, E>: Sized + Send + Sync + 'static
    where C: Send + 'static,
          E: Send + 'static,
{
    fn new() -> Self;
    fn next(&self) -> Option<Interpret<C, E>>;

    fn run(tx: Sender<Interpret<C, E>>, rx: Receiver<E>) {
        let gateway = Arc::new(Self::new());
        let global  = gateway.clone();

        thread::spawn(move || {
            loop {
                gateway.next()
                       .map(|i| {
                           tx.send(i)
                             .map_err(|err| error!("Error sending command: {:?}", err))
                       });
            }
        });

        thread::spawn(move || {
            loop {
                match rx.recv() {
                    Ok(e)    => global.pulse(e),
                    Err(err) => error!("Error receiving event: {:?}", err),
                }
            }
        });
    }

    #[allow(unused_variables)]
    fn pulse(&self, e: E) {
        // ignore global events by default
    }
}
