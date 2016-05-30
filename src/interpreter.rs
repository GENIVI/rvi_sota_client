use std::borrow::Cow;
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Sender, Receiver, channel};

use datatype::{AccessToken, Auth, ClientId, ClientSecret, Command, Config,
               Error, Event, UpdateState};
use datatype::Command::*;
use http_client::AuthClient;
use interaction_library::gateway::Interpret;
use oauth2::authenticate;
use ota_plus::OTA;


pub type Wrapped = Interpret<Command, Event>;


pub struct Env<'t> {
    pub config:       Config,
    pub access_token: Option<Cow<'t, AccessToken>>,
    pub wtx:          Sender<Wrapped>,
}


pub trait Interpreter<Env, I, O> {
    fn interpret(env: &mut Env, msg: I, otx: Sender<O>);

    fn run(env: &mut Env, irx: Receiver<I>, otx: Sender<O>) {
        loop {
            match irx.recv() {
                Ok(msg)  => Self::interpret(env, msg, otx.clone()),
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

pub struct AuthenticationRetrier;

impl Interpreter<(), Event, Command> for AuthenticationRetrier {
    fn interpret(_: &mut (), event: Event, ctx: Sender<Command>) {
        match event {
            Event::NotAuthenticated => {
                info!("Trying to authenticate again");
                let _ = ctx.send(Command::Authenticate(None));
            }
            _                       => {}
        }
    }
}

/* TODO: Handle events to PackageManager
pub struct AutoPackageInstaller;

impl Interpreter<(), Event, Command> for AutoPackageInstaller {
    fn interpret(env: &mut Env, event: Event, ctx: Sender<Command>) {
        match event {
            Event::DownloadComplete => {
                match env.config.ota.package_manager.install_package(p) {
                    _ => {
                        let _ = ctx.send(Command::UpdateReport());
                    }
                }
            }
            Event::GetInstalledSoftware => {
                match env.config.ota.package_manager.installed_packages() {
                    _ => {
                        let _ = ctx.send(Command::InstalledSoftware());
                    }
                }
            }
            _ => {}
        }
    }
}
*/


pub struct GlobalInterpreter;

impl<'t> Interpreter<Env<'t>, Wrapped, Event> for GlobalInterpreter {
    fn interpret(env: &mut Env, w: Wrapped, global_tx: Sender<Event>) {
        info!("Interpreting: {:?}", w.cmd);
        let (multi_tx, multi_rx): (Sender<Event>, Receiver<Event>) = channel();
        let local_tx = w.etx.clone();
        let w_clone  = w.clone();

        let _ = command_interpreter(env, w.cmd, multi_tx)
            .map_err(|err| {
                if let Error::AuthorizationError(_) = err {
                    debug!("retry authorization and request");
                    let _ = env.wtx.send(Wrapped {
                        cmd: Command::Authenticate(None),
                        etx: None
                    });
                    let _ = env.wtx.send(w_clone);
                } else {
                    let ev = Event::Error(format!("{}", err));
                    let _  = global_tx.send(ev.clone()).unwrap();
                    send(ev, &local_tx);
                }
            })
            .map(|_| {
                let mut last_ev = None;
                for ev in multi_rx {
                    let _ = global_tx.send(ev.clone()).unwrap();
                    last_ev = Some(ev);
                }
                match last_ev {
                    Some(ev) => send(ev, &local_tx),
                    None     => panic!("no event to send back")
                }
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
        Some(token) => match token {
            Cow::Borrowed(t) => {
                let client = AuthClient::new(Auth::Token(t.clone()));
                authenticated(env, &client, cmd, etx)
            }

            Cow::Owned(t) => {
                let client = AuthClient::new(Auth::Token(t));
                authenticated(env, &client, cmd, etx)
            }
        },

        None => {
            let client = AuthClient::new(Auth::Credentials(
                ClientId(env.config.auth.client_id.clone()),
                ClientSecret(env.config.auth.secret.clone())
            ));
            unauthenticated(env, client, cmd, etx)
        }
    }
}

fn authenticated(env: &mut Env, client: &AuthClient, cmd: Command, etx: Sender<Event>) -> Result<(), Error> {
    let mut ota = OTA::new(client, env.config.clone());

    match cmd {
        Authenticate(_) => try!(etx.send(Event::Ok)),

        AcceptUpdate(ref id) => {
            try!(etx.send(Event::UpdateStateChanged(id.clone(), UpdateState::Downloading)));
            let report = try!(ota.install_package_update(&id, &etx));
            try!(ota.send_install_report(&report));
            info!("Update finished. Report sent: {:?}", report)
        }

        GetPendingUpdates => {
            let mut updates = try!(ota.get_package_updates());
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
            try!(ota.update_installed_packages());
            try!(etx.send(Event::Ok));
            info!("Posted installed packages to the server.")
        }

        Shutdown => exit(0),
    }

    Ok(())
}

fn unauthenticated(env: &mut Env, client: AuthClient, cmd: Command, etx: Sender<Event>) -> Result<(), Error> {
    match cmd {
        Authenticate(_) => {
            let token = try!(authenticate(&env.config.auth, &client));
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
