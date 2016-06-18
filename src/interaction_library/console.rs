use chan;
use chan::{Sender, Receiver};
use std::{io, thread};
use std::fmt::Debug;
use std::io::Write;
use std::str::FromStr;
use std::string::ToString;
use std::sync::{Arc, Mutex};

use super::broadcast::Broadcast;
use super::gateway::{Gateway, Interpret};


pub struct Console;

impl<C, E> Gateway<C, E> for Console
    where C: FromStr  + Send + Clone + Debug + 'static,
          E: ToString + Send + Clone + Debug + 'static,
          <C as FromStr>::Err: Debug,
{
    fn new(itx: Sender<Interpret<C, E>>, shutdown_rx: Receiver<()>) -> Self {
        let mut shutdown = Broadcast::new(shutdown_rx);
        let (etx, erx)   = chan::sync::<E>(0);
        let etx          = Arc::new(Mutex::new(etx));

        let stop_tx = shutdown.subscribe();
        thread::spawn(move || {
            loop {
                chan_select! {
                    default => match parse_input(get_input()) {
                        Ok(cmd)  => itx.send(Interpret{ command: cmd, response_tx: Some(etx.clone()) }),
                        Err(err) => println!("(error): {:?}", err)
                    },
                    stop_tx.recv() => break
                }
            }
        });

        let stop_rx = shutdown.subscribe();
        thread::spawn(move || {
            loop {
                chan_select! {
                    erx.recv() -> e => match e {
                        Some(e) => println!("(event): {}", e.to_string()),
                        None    => panic!("all console event transmitters are closed")
                    },
                    stop_rx.recv()  => break
                }
            }
        });

        println!("OTA Plus Client REPL started.");
        Console
    }
}

fn get_input() -> String {
    let mut input = String::new();
    let _ = io::stdout().write("> ".as_bytes());
    let _ = io::stdin().read_line(&mut input);
    input
}

fn parse_input<C: FromStr>(s: String) -> Result<C, <C as FromStr>::Err> {
    s.parse()
}
