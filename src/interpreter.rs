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


#[derive(Clone)]
pub struct Env<'a> {
    pub config:       Config,
    pub access_token: Option<Cow<'a, AccessToken>>,
    pub http_client:  Arc<Mutex<HttpClient>>,
}


pub struct OurInterpreter;

impl<'a> Interpreter<Env<'a>, Command, Event> for OurInterpreter {
    fn interpret(env: &mut Env, original: &Env, cmd: Command, tx: Sender<Event>) {
        info!("Interpreting: {:?}", cmd);
        interpreter(env, original, cmd, tx.clone())
            .unwrap_or_else(|err| {
                tx.send(Event::Error(format!("{}", err)))
                    .unwrap_or_else(|_| error!("interpret: send failed"))
            })
    }
}


pub struct AutoAcceptor;

impl Interpreter<(), Event, Command> for AutoAcceptor {
    fn interpret(_: &mut (), _: &(), e: Event, ctx: Sender<Command>) {
        fn f(e: &Event, ctx: Sender<Command>) {
            if let &Event::NewUpdateAvailable(ref id) = e {
                let _ = ctx.send(Command::AcceptUpdate(id.clone()));
            }
        }

        info!("Event interpreter: {:?}", e);
        match e {
            Event::Batch(ref evs) => {
                for ev in evs {
                    f(&ev, ctx.clone())
                }
            }
            e => f(&e, ctx)
        }
    }
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

fn interpreter(env: &mut Env, original: &Env, cmd: Command, tx: Sender<Event>) -> Result<(), Error> {
    let client_clone = env.http_client.clone();

    if let Authenticate(credentials) = cmd {
        match credentials {
            Some(cc) => {
                env.config.auth.client_id = cc.id.get.to_owned();
                env.config.auth.secret = cc.secret.get.to_owned();

                let mut client = try!(client_clone.lock());
                match authenticate(&env.config.auth, &mut *client) {
                    Ok(token) => {
                        env.access_token = Some(token.into());
                    },
                    Err(err) => return Err(err)
                }
            },
            None => {
                env.config.auth.client_id = original.config.auth.client_id.to_owned();
                env.config.auth.secret = original.config.auth.secret.to_owned();
            }
        }
        return Ok(())
    }

    match env.access_token.to_owned() {
        Some(token) => Ok({
            partial_apply!(
                [get_package_updates, update_installed_packages],
                [send_install_report],
                [install_package_update],
                &env.config, client_clone, &token
            );

            match cmd {
                AcceptUpdate(ref id) => {
                    try!(tx.send(Event::UpdateStateChanged(id.clone(), UpdateState::Downloading)));
                    let report = try!(install_package_update(id.to_owned(), tx.to_owned()));
                    try!(send_install_report(report.clone()));
                    info!("Update finished. Report sent: {:?}", report)
                }

                Authenticate(_) => unreachable!(),

                GetPendingUpdates => {
                    let mut updates = try!(get_package_updates());
                    updates.sort_by_key(|e| e.createdAt.clone());
                    let update_events: Vec<Event> = updates
                        .iter()
                        .map(|u| Event::NewUpdateAvailable(u.id.clone()))
                        .collect();
                    info!("New package updates available: {:?}", update_events);
                    try!(tx.send(Event::Batch(update_events)))
                }

                ListInstalledPackages => {
                    let pkgs = try!(env.config.ota.package_manager.installed_packages());
                    try!(tx.send(Event::FoundInstalledPackages(pkgs.clone())))
                }

                Shutdown => exit(0),

                UpdateInstalledPackages => {
                    try!(update_installed_packages());
                    info!("Posted installed packages to the server.")
                }
            }
        }),

        None => Ok({
            match cmd {
                Authenticate(_)         => unreachable!(),
                Shutdown                => exit(0),

                AcceptUpdate(_)       |
                GetPendingUpdates     |
                ListInstalledPackages |
                UpdateInstalledPackages => {
                    tx.send(Event::NotAuthenticated)
                        .unwrap_or_else(|_| error!("interpreter: send failed."))
                }
            }
        })
    }
}
