use std::sync::mpsc::Sender;

use datatype::{AccessToken, Command, Config, Event};
use package_manager::PackageManager;
use interaction_library::interpreter::Interpreter;


pub struct OurInterpreter;

#[allow(dead_code)]
pub struct Env<'a> {
    config:       &'a Config,
    access_token: Option<&'a AccessToken>,
    pkg_manager:  &'a PackageManager,
}


impl<'a> Interpreter<Env<'a>, Command, Event> for OurInterpreter {

    #[allow(unused_variables)]
    fn interpret(env: &Env, cmd: Command, rx: Sender<Event>) {
        match cmd {
            Command::GetPendingUpdates     => unimplemented!(),
            Command::PostInstalledPackages => unimplemented!(),
            Command::AcceptUpdate(ref id)  => unimplemented!(),
            Command::ListInstalledPackages => unimplemented!(),
            Command::Shutdown              => unimplemented!(),
        }
    }

}
