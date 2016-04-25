use std::borrow::Cow;
use std::process::exit;
use std::sync::Arc;
use std::sync::mpsc::Sender;

use auth_plus::authenticate;
use datatype::{AccessToken, Command, Config, Error, Event, UpdateReport,
               UpdateRequestId, UpdateState, UpdateResultCode};
use datatype::Command::*;
use http_client::HttpClient2;
use interaction_library::interpreter::Interpreter;
use ota_plus::{get_package_updates, download_package_update, post_packages, send_install_report};


#[derive(Clone)]
pub struct Env<'a> {
    pub config:       Config,
    pub access_token: Option<Cow<'a, AccessToken>>,
    pub http_client:  Arc<HttpClient2>,
}

// This macro partially applies the config and http client to the passed
// in functions.
macro_rules! partial_apply {
    ([ $( $fun0: ident ),* ], [ $( $fun1: ident ),* ], [ $( $fun2: ident ),* ],  $env: expr, $token: expr) => {
        $(let $fun0 = ||           $fun0(&$env.config, &*$env.http_client, $token);)*;
        $(let $fun1 = |arg|        $fun1(&$env.config, &*$env.http_client, $token, &arg);)*;
        $(let $fun2 = |arg1, arg2| $fun2(&$env.config, &*$env.http_client, $token, &arg1, &arg2);)*;
    }
}

// XXX: Move this somewhere else?
fn install_package_update(config:      &Config,
                          http_client: &HttpClient2,
                          token:       &AccessToken,
                          id:          &UpdateRequestId,
                          tx:          &Sender<Event>) -> Result<UpdateReport, Error> {

    match download_package_update(config, http_client, token, id) {

        Ok(path) => {
            info!("Downloaded at {:?}. Installing...", path);
            try!(tx.send(Event::UpdateStateChanged(id.clone(), UpdateState::Installing)));

            let p = try!(path.to_str()
                         .ok_or(Error::ParseError(format!("Path is not valid UTF-8: {:?}", path))));

            match config.ota.package_manager.install_package(p) {

                Ok((code, output)) => {
                    try!(tx.send(Event::UpdateStateChanged(id.clone(), UpdateState::Installed)));

                    // XXX: Slight code duplication, see interpret(PostInstalledPackages).
                    let pkgs = try!(config.ota.package_manager.installed_packages());
                    try!(post_packages(config, http_client, token, &pkgs));

                    Ok(UpdateReport::new(id.clone(), code, output))
                }

                Err((code, output)) => {
                    try!(tx.send(Event::UpdateErrored(id.clone(), format!("{:?}: {:?}", code, output))));
                    Ok(UpdateReport::new(id.clone(), code, output))
                }

            }

        }

        Err(err) => {
            try!(tx.send(Event::UpdateErrored(id.clone(), format!("{:?}", err))));
            Ok(UpdateReport::new(id.clone(),
                              UpdateResultCode::GENERAL_ERROR,
                              format!("Download failed: {:?}", err)))
        }
    }

}

fn interpreter(env: &mut Env, cmd: Command, tx: &Sender<Event>) -> Result<(), Error> {

    Ok(if let Some(token) = env.access_token.to_owned() {

        partial_apply!(
            [get_package_updates],
            [post_packages, send_install_report],
            [install_package_update], &env, &token);

        match cmd {

            Authenticate(_)       => (), // Already authenticated.

            AcceptUpdate(ref id)  => {
                try!(tx.send(Event::UpdateStateChanged(id.clone(), UpdateState::Downloading)));
                let report = try!(install_package_update(id.to_owned(), tx.to_owned()));
                try!(send_install_report(report.clone()));
                info!("Update finished. Report sent: {:?}", report)
            }

            GetPendingUpdates     => {
                let updates = try!(get_package_updates());
                let update_events: Vec<Event> = updates
                    .iter()
                    .map(|id| Event::NewUpdateAvailable(id.clone()))
                    .collect();
                info!("New package updates available: {:?}", update_events);
                try!(tx.send(Event::Batch(update_events)))
            }

            ListInstalledPackages => {
                let pkgs = try!(env.config.ota.package_manager.installed_packages());
                try!(tx.send(Event::FoundInstalledPackages(pkgs.clone())))
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
                // XXX: partially apply?
                let token = try!(authenticate(&env.config.auth, &*env.http_client));
                env.access_token = Some(token.into())
            }

            Shutdown                      => exit(0),

            AcceptUpdate(_)       |
            GetPendingUpdates     |
            ListInstalledPackages |
            PostInstalledPackages         =>
                tx.send(Event::NotAuthenticated)
                  .unwrap_or(error!("not_auth: send failed."))
            }

    })

}

pub struct OurInterpreter;

impl<'a> Interpreter<Env<'a>, Command, Event> for OurInterpreter {

    fn interpret(env: &mut Env, cmd: Command, tx: Sender<Event>) {
        interpreter(env, cmd, &tx)
            .unwrap_or_else(|err| tx.send(Event::Error(format!("{}", err)))
                            .unwrap_or(error!("interpret: send failed.")))
    }

}
