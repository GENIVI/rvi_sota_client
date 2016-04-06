use std::io;
use std::str::FromStr;
use std::sync::mpsc::{Sender, Receiver};

use std::thread;

use datatype::{Command, Event};

impl FromStr for Command {
    type Err = ();
    fn from_str(s: &str) -> Result<Command, ()> {
        match s {
            "GetPendingUpdates" => Ok(Command::GetPendingUpdates),
            "PostInstalledPackages" => Ok(Command::PostInstalledPackages),
            "ListInstalledPackages" => Ok(Command::ListInstalledPackages),
            _              => Err(()),
        }
    }
}

pub fn start(erx: Receiver<Event>, ctx: Sender<Command>) {
    let _ = thread::Builder::new().name("REPL Print loop".to_string()).spawn(move || {
        loop {
            println!("# => {:?}", erx.recv().unwrap());
        }
    });

    println!("Ota Plus Client REPL started.");
    loop {
        let mut input = String::new();
        print!("> ");
        let _ = io::stdin().read_line(&mut input);

        let _ = input.trim().parse().map(|cmd| ctx.send(cmd));
    }
}
