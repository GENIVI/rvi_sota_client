use std::sync::mpsc::Sender;

use datatype::{AccessToken, Command, Config, Event};
use datatype::Command::*;
use package_manager::PackageManager;
use http_client::HttpClient2;
use interaction_library::interpreter::Interpreter;


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
            AcceptUpdate(ref id)  => unimplemented!(),
            GetPendingUpdates     => unimplemented!(),
            ListInstalledPackages => unimplemented!(),
            PostInstalledPackages => unimplemented!(),
            Shutdown              => unimplemented!(),
        }
    }

/*
    fn get_installed_packages(&self) -> Result<Vec<Package>, Error> {
        self.config.ota.package_manager.installed_packages()
    }
    fn post_installed_packages(&self) {
        let _ = self.get_installed_packages().and_then(|pkgs| {
            debug!("Found installed packages in the system: {:?}", pkgs);
            post_packages::<C>(&self.token, &self.config, &pkgs)
        }).map(|_| {
            info!("Posted installed packages to the server.");
        }).map_err(|e| {
            error!("Error fetching/posting installed packages: {:?}.", e);
        });
    }
*/

}
