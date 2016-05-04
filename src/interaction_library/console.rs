use std::{io, thread};
use std::fmt::Debug;
use std::str::FromStr;
use std::string::ToString;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};

use super::gateway::{Gateway, Interpret};


pub struct Console;

impl<C, E> Gateway<C, E> for Console
    where C: FromStr + Send + 'static,
          E: ToString + Send + 'static,
          <C as FromStr>::Err: Debug,
{
    fn new() -> Console {
        Console
    }

    fn next(&self) -> Option<Interpret<C,E>> {
        match parse_input(&get_input()) {
            Ok(cmd) => {
                let (etx, erx): (Sender<E>, Receiver<E>) = mpsc::channel();
                thread::spawn(move || {
                    match erx.recv() {
                        Ok(event) => println!("{}", event.to_string()),
                        Err(err)  => error!("Error receiving event: {:?}", err),
                    }
                });
                Some(Interpret{
                    cmd: cmd,
                    etx: etx,
                })
            }

            Err(err) => {
                println!("{:?}", err);
                None
            }
        }
    }

    fn pulse(&self, e: E) {
        println!("(global): {}", e.to_string());
    }
}

fn get_input() -> String {
    print!("> ");
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
    input
}

fn parse_input<C: FromStr>(s: &str) -> Result<C, <C as FromStr>::Err> {
    s.parse()
}
