use std::borrow::Cow;
use std::process::exit;
use std::sync::mpsc::Sender;

use datatype::{AccessToken, Command, Config, Error, Event, UpdateState};
use datatype::Command::*;
use package_manager::PackageManager;
use http_client::HttpClient2;
use interaction_library::interpreter::Interpreter;
use new_auth_plus::authenticate;
use new_ota_plus::{get_package_updates, download_package_update,
                   post_packages, send_install_report};


#[allow(dead_code)]
pub struct Env<'a> {
    config:       &'a Config,
    access_token: Option<Cow<'a, AccessToken>>,
    pkg_manager:  &'a PackageManager,
    http_client:  &'a HttpClient2,
}

macro_rules! fun0 {
    ($fun: ident, $env: expr, $token: expr) =>
        (let $fun = || $fun($env.config, $env.http_client, &$token));
}

macro_rules! fun1 {
    ($fun: ident, $env: expr, $token: expr) =>
        (let $fun = |arg| $fun($env.config, $env.http_client, &$token, &arg));
}

fn interpreter(env: &mut Env, cmd: Command, rx: &Sender<Event>) -> Result<(), Error> {

    Ok(if let Some(token) = env.access_token.to_owned() {

        fun0!(get_package_updates,     &env, &token);
        fun1!(post_packages,           &env, &token);
        fun1!(download_package_update, &env, &token);
        fun1!(send_install_report,     &env, &token);

        match cmd {
            Authenticate(_)       => (),
            AcceptUpdate(ref id)  => {

                try!(rx.send(Event::UpdateStateChanged(id.clone(), UpdateState::Downloading)));

            }
/*
    fn accept_update(&self, id: &UpdateRequestId) {
        let report = download_package_update::<C>(&self.token, &self.config, id)
            .and_then(|path| {
                info!("Downloaded at {:?}. Installing...", path);
                self.publish(Event::UpdateStateChanged(id.clone(), UpdateState::Installing));

                let p = try!(path.to_str().ok_or(Error::ParseError(format!("Path is not valid UTF-8: {:?}", path))));
                self.config.ota.package_manager.install_package(p)
                    .map(|(code, output)| {
                        self.publish(Event::UpdateStateChanged(id.clone(), UpdateState::Installed));
                        UpdateReport::new(id.clone(), code, output)
                    })
                    .or_else(|(code, output)| {
                        self.publish(Event::UpdateErrored(id.clone(), format!("{:?}: {:?}", code, output)));
                        Ok(UpdateReport::new(id.clone(), code, output))
                    })
            }).unwrap_or_else(|e| {
                self.publish(Event::UpdateErrored(id.clone(), format!("{:?}", e)));
                UpdateReport::new(id.clone(),
                                   UpdateResultCode::GENERAL_ERROR,
                                   format!("Download failed: {:?}", e))
            });

        match send_install_report::<C>(&self.token, &self.config, &report) {
            Ok(_) => info!("Update finished. Report sent: {:?}", report),
            Err(e) => error!("Error reporting back to the server: {:?}", e)
        }
    }
*/

            GetPendingUpdates     => {
                let updates = try!(get_package_updates());
                let update_events: Vec<Event> = updates
                    .iter()
                    .map(|id| Event::NewUpdateAvailable(id.clone()))
                    .collect();
                info!("New package updates available: {:?}", update_events);
                try!(rx.send(Event::Batch(update_events)))
            }

            ListInstalledPackages => {
                let pkgs = try!(env.config.ota.package_manager.installed_packages());
                try!(rx.send(Event::FoundInstalledPackages(pkgs.clone())))
            }

            PostInstalledPackages => {
                let pkgs = try!(env.config.ota.package_manager.installed_packages());
                debug!("Found installed packages in the system: {:?}", pkgs);
                try!(post_packages(pkgs));
                info!("Posted installed packages to the server.")
            }

            Shutdown              => exit(0)
        }

    } else {

        match cmd {

            Authenticate(_)               => {
                let token = try!(authenticate(&env.config.auth, env.http_client));
                env.access_token = Some(token.into())
            }

            Shutdown                      => exit(0),

            AcceptUpdate(_)       |
            GetPendingUpdates     |
            ListInstalledPackages |
            PostInstalledPackages         =>
                rx.send(Event::NotAuthenticated)
                  .unwrap_or(error!("not_auth: send failed."))
            }

    })

}

pub struct OurInterpreter;

impl<'a> Interpreter<Env<'a>, Command, Event> for OurInterpreter {

    fn interpret(env: &mut Env, cmd: Command, rx: Sender<Event>) {
        interpreter(env, cmd, &rx)
            .unwrap_or_else(|err| rx.send(Event::Error(format!("{}", err)))
                            .unwrap_or(error!("interpret: send failed.")))
    }

}
