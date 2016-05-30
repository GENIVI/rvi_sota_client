use std::{io, thread};
use std::fmt::Debug;
use std::str::FromStr;
use std::string::ToString;
use std::sync::{Arc, Mutex, mpsc};
use std::sync::mpsc::{Sender, Receiver};

use super::gateway::{Gateway, Interpret};


pub struct Console<E> {
    etx: Arc<Mutex<Sender<E>>>
}

impl<C: Clone, E: Clone> Gateway<C, E> for Console<E>
    where C: Send + FromStr  + 'static,
          E: Send + ToString + 'static,
          <C as FromStr>::Err: Debug,
{
    fn new() -> Console<E> {
        let (etx, erx): (Sender<E>, Receiver<E>) = mpsc::channel();

        thread::spawn(move || {
            loop {
                match erx.recv() {
                    Ok(event) => println!("{}", event.to_string()),
                    Err(err)  => error!("Error receiving event: {:?}", err),
                }
            }
        });

        Console { etx: Arc::new(Mutex::new(etx)) }
    }

    fn next(&self) -> Option<Interpret<C,E>> {
        match parse_input(get_input()) {
            Ok(cmd)  => Some(Interpret{
                cmd: cmd,
                etx: Some(self.etx.clone()),
            }),
            Err(err) => {
                println!("{:?}", err);
                None
            }
        }
    }
}

fn get_input() -> String {
    print!("> ");
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
    input
}

fn parse_input<C: FromStr>(s: String) -> Result<C, <C as FromStr>::Err> {
    s.parse()
}
