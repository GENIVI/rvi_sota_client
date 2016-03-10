use std::io;
use std::str::FromStr;

enum Command {
    ListPackages,
}

impl FromStr for Command {
    type Err = ();
    fn from_str(s: &str) -> Result<Command, ()> {
        match s {
            "ListPackages" => Ok(Command::ListPackages),
            _              => Err(()),
        }
    }
}

fn interpret(cmd: Command) {
    match cmd {
        Command::ListPackages => info!("ok"),
    };
}

pub fn read_interpret_loop() {

    loop {

        let mut input = String::new();
        let _ = io::stdin().read_line(&mut input);

        match input.trim().parse() {
            Ok(cmd) => interpret(cmd),
            Err(_)  => error!("Parse error."),
        };

    }

}
