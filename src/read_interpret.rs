use std::io;
use std::str::FromStr;

use package_manager::PackageManager;
use datatype::Command;

pub struct ReplEnv<M: PackageManager> {
    package_manager: M,
}

impl<M: PackageManager> ReplEnv<M> {
    pub fn new(manager: M) -> ReplEnv<M> {
        ReplEnv { package_manager: manager }
    }
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

fn list_packages<M>(_: &M)
    where M: PackageManager {
        unimplemented!();
}

fn interpret<M>(env: &ReplEnv<M>, cmd: Command)
    where M: PackageManager {
    match cmd {
        Command::ListPackages => list_packages(&env.package_manager),
        _ => {}
    };
}

pub fn read_interpret_loop<M>(env: ReplEnv<M>)
    where M: PackageManager {

    loop {

        let mut input = String::new();
        let _ = io::stdin().read_line(&mut input);

        let _ = input.trim().parse().map(|cmd| interpret(&env, cmd));

    }
}
