use std::borrow::Cow;
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;

use auth_plus::authenticate;
use datatype::{AccessToken, Command, Config, Error, Event, UpdateState};
use datatype::Command::*;
use http_client::HttpClient;
use interaction_library::interpreter::Interpreter;
use ota_plus::{get_package_updates, install_package_update,
               update_installed_packages, send_install_report};


pub struct Env<'a> {
    pub config:       Config,
    pub access_token: Option<Cow<'a, AccessToken>>,
    pub http_client:  Arc<Mutex<HttpClient>>,
}

// This macro partially applies the config, http client and token to the
// passed in functions.
macro_rules! partial_apply {
    ([ $( $fun0: ident ),* ], // Functions of arity 0,
     [ $( $fun1: ident ),* ], // arity 1,
     [ $( $fun2: ident ),* ], // and arity 2.
     $config: expr, $client: expr, $token: expr) => {
        $(let $fun0 = ||           $fun0(&$config, &mut *$client.lock().unwrap(), $token);)*
        $(let $fun1 = |arg|        $fun1(&$config, &mut *$client.lock().unwrap(), $token, &arg);)*
        $(let $fun2 = |arg1, arg2| $fun2(&$config, &mut *$client.lock().unwrap(), $token, &arg1, &arg2);)*
    }
}

fn interpreter(env: &mut Env, cmd: Command, tx: &Sender<Event>) -> Result<(), Error> {

    Ok(if let Some(token) = env.access_token.to_owned() {

        let client_clone = env.http_client.clone();

        partial_apply!(
            [get_package_updates, update_installed_packages],
            [send_install_report],
            [install_package_update], &env.config, client_clone, &token);

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

            UpdateInstalledPackages => {
                try!(update_installed_packages());
                info!("Posted installed packages to the server.")
            }

            Shutdown              => exit(0)
        }

    } else {

        match cmd {

            Authenticate(_)               => {
                // XXX: partially apply?
                let client_clone = env.http_client.clone();
                let mut client = client_clone.lock().unwrap();
                let token = try!(authenticate(&env.config.auth, &mut *client));
                env.access_token = Some(token.into())
            }

            Shutdown                      => exit(0),

            AcceptUpdate(_)       |
            GetPendingUpdates     |
            ListInstalledPackages |
            UpdateInstalledPackages         =>
                tx.send(Event::NotAuthenticated)
                  .unwrap_or(error!("interpreter: send failed."))
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
