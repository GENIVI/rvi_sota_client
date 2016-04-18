use std::sync::mpsc::Sender;

use datatype::{AccessToken, Command, Config, Event};
use datatype::Command::*;
use package_manager::PackageManager;
use http_client::HttpClient2;
use interaction_library::interpreter::Interpreter;
use interaction_library::interpreter;
use package_manager::PackageManager;


pub struct OurInterpreter;

#[allow(dead_code)]
pub struct Env<'a> {
    config:       &'a Config,
    access_token: Option<&'a AccessToken>,
    pkg_manager:  &'a PackageManager,
    http_client:  &'a HttpClient2,
}


impl<'a> Interpreter<Env<'a>, Command, Event> for OurInterpreter {

    #[allow(unused_variables)]
    fn interpret(env: &Env, cmd: Command, rx: Sender<Event>) {
        match cmd {
            GetPendingUpdates     => unimplemented!(),
            PostInstalledPackages => unimplemented!(),
            AcceptUpdate(ref id)  => unimplemented!(),
            ListInstalledPackages => unimplemented!(),
            Shutdown              => unimplemented!(),
        }
    }

}
