use chan;
use chan::Sender;
use std::{io, thread};
use std::fmt::Debug;
use std::io::Write;
use std::str::FromStr;
use std::string::ToString;
use std::sync::{Arc, Mutex};

use super::gateway::{Gateway, Interpret};


pub struct Console;

impl<C, E> Gateway<C, E> for Console
    where C: FromStr  + Send + Clone + Debug + 'static,
          E: ToString + Send + Clone + Debug + 'static,
          <C as FromStr>::Err: Debug,
{
    fn new(itx: Sender<Interpret<C, E>>) -> Result<Self, String> {
        let (etx, erx) = chan::sync::<E>(0);
        let etx        = Arc::new(Mutex::new(etx));

        thread::spawn(move || {
            loop {
                match parse_input(get_input()) {
                    Ok(cmd)  => itx.send(Interpret{ command: cmd, response_tx: Some(etx.clone()) }),
                    Err(err) => error!("Console Error: {:?}", err)
                }
            }
        });

        thread::spawn(move || {
            loop {
                match erx.recv() {
                    Some(e) => info!("Console Response: {}", e.to_string()),
                    None    => panic!("all console event transmitters are closed")
                }
            }
        });

        println!("OTA Plus Client REPL started.");
        Ok(Console)
    }
}

fn get_input() -> String {
    let mut input = String::new();
    let _ = io::stdout().write("> ".as_bytes());
    io::stdout().flush().expect("couldn't flush console stdout buffer");
    let _ = io::stdin().read_line(&mut input);
    input
}

fn parse_input<C: FromStr>(s: String) -> Result<C, <C as FromStr>::Err> {
    s.parse()
}
