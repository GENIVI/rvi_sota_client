use std::borrow::Cow;
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Sender, Receiver, channel};

use auth_plus::authenticate;
use datatype::{AccessToken, Command, Config, Error, Event, UpdateState};
use datatype::Command::*;
use http_client::HttpClient;
use interaction_library::gateway::Interpret;
use ota_plus::{get_package_updates, install_package_update, update_installed_packages,
               send_install_report};


#[derive(Clone)]
pub struct Env<'a> {
    pub config: Config,
    pub access_token: Option<Cow<'a, AccessToken>>,
    pub http_client: Arc<Mutex<HttpClient>>,
}


pub trait Interpreter<Env, I, O> {
    fn interpret(env: &mut Env, msg: I, otx: Sender<O>);

    fn run(env: &mut Env, irx: Receiver<I>, otx: Sender<O>) {
        loop {
            match irx.recv() {
                Ok(msg) => Self::interpret(env, msg, otx.clone()),
                Err(err) => error!("Error receiving command: {:?}", err),
            }
        }
    }
}


pub struct AutoAcceptor;

impl Interpreter<(), Event, Command> for AutoAcceptor {
    fn interpret(_: &mut (), event: Event, ctx: Sender<Command>) {
        info!("Automatic interpreter: {:?}", event);
        match event {
            Event::Batch(ref evs) => {
                for ev in evs {
                    accept(&ev, ctx.clone())
                }
            }
            ev => accept(&ev, ctx),
        }

        fn accept(event: &Event, ctx: Sender<Command>) {
            if let &Event::NewUpdateAvailable(ref id) = event {
                let _ = ctx.send(Command::AcceptUpdate(id.clone()));
            }
        }
    }
}


pub struct GlobalInterpreter;

impl<'a> Interpreter<Env<'a>, Interpret<Command, Event>, Event> for GlobalInterpreter {
    fn interpret(env: &mut Env, i: Interpret<Command, Event>, etx: Sender<Event>) {
        info!("Interpreting: {:?}", i.cmd);
        let (multi_tx, multi_rx): (Sender<Event>, Receiver<Event>) = channel();
        let local_tx = i.etx.clone();

        let _ = command_interpreter(env, i.cmd, multi_tx)
            .map_err(|err| {
                let ev = Event::Error(format!("{}", err));
                let _  = etx.send(ev.clone()).unwrap();
                send(ev, &local_tx);
            })
            .map(|_| {
                let mut last_ev: Event;
                for ev in multi_rx {
                    let _ = etx.send(ev.clone()).unwrap();
                    last_ev = ev;
                }
                send(last_ev, &local_tx);
            });

        fn send(ev: Event, local_tx: &Option<Arc<Mutex<Sender<Event>>>>) {
            // unwrap failed sends to avoid receiver thread deadlocking
            if let Some(ref local) = *local_tx {
                let _ = local.lock().unwrap().send(ev).unwrap();
            }
        }
    }
}

fn command_interpreter(env: &mut Env, cmd: Command, etx: Sender<Event>) -> Result<(), Error> {
    match env.access_token.to_owned() {
        Some(ref token) => authenticated(env, cmd, etx, token),
        None            => unauthenticated(env, cmd, etx),
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

fn authenticated<'a>(env: &mut Env, cmd: Command, etx: Sender<Event>, token: &Cow<'a, AccessToken>)
                     -> Result<(), Error> {

    let client = env.http_client.clone();
    partial_apply!([get_package_updates, update_installed_packages],
                   [send_install_report],
                   [install_package_update],
                   &env.config, client, &token);

    match cmd {
        Authenticate(_) => (),

        AcceptUpdate(ref id) => {
            try!(etx.send(Event::UpdateStateChanged(id.clone(), UpdateState::Downloading)));
            let report = try!(install_package_update(id.to_owned(), etx.to_owned()));
            try!(send_install_report(report.clone()));
            info!("Update finished. Report sent: {:?}", report)
        }

        GetPendingUpdates => {
            let mut updates = try!(get_package_updates());
            updates.sort_by_key(|e| e.installPos);
            let evs: Vec<Event> = updates.iter()
                                         .map(|up| Event::NewUpdateAvailable(up.requestId.clone()))
                                         .collect();
            info!("New package updates available: {:?}", evs);
            try!(etx.send(Event::Batch(evs)))
        }

        ListInstalledPackages => {
            let pkgs = try!(env.config.ota.package_manager.installed_packages());
            try!(etx.send(Event::FoundInstalledPackages(pkgs.clone())))
        }

        UpdateInstalledPackages => {
            try!(update_installed_packages());
            try!(etx.send(Event::Ok));
            info!("Posted installed packages to the server.")
        }

        Shutdown => exit(0),
    }

    Ok(())
}

fn unauthenticated(env: &mut Env, cmd: Command, etx: Sender<Event>) -> Result<(), Error> {
    match cmd {
        Authenticate(_) => {
            let client_clone = env.http_client.clone();
            let mut client = client_clone.lock().unwrap();
            let token = try!(authenticate(&env.config.auth, &mut *client));
            env.access_token = Some(token.into());
            try!(etx.send(Event::Ok));
        }

        AcceptUpdate(_)       |
        GetPendingUpdates     |
        ListInstalledPackages |
        UpdateInstalledPackages => try!(etx.send(Event::NotAuthenticated)),

        Shutdown => exit(0),
    }

    Ok(())
}
