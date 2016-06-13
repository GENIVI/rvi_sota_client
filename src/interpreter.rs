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


pub trait Interpreter<I, O> {
    fn interpret(&mut self, msg: I, otx: Sender<O>);

    fn run(&mut self, irx: Receiver<I>, otx: Sender<O>) {
        loop {
            match irx.recv() {
                Ok(msg)  => self.interpret(msg, otx.clone()),
                Err(err) => error!("Error receiving command: {:?}", err),
            }
        }
    }
}


pub struct EventInterpreter;

impl Interpreter<Event, Command> for EventInterpreter {
    fn interpret(&mut self, event: Event, ctx: Sender<Command>) {
        info!("Event interpreter: {:?}", event);
        let _ = match event {
            Event::NotAuthenticated => {
                debug!("trying to authenticate again...");
                ctx.send(Command::Authenticate(None))
            }

            Event::NewUpdateAvailable(ref id) => {
                ctx.send(Command::AcceptUpdate(id.clone()))
            }

            /* TODO: Handle PackageManger events
            Event::DownloadComplete => {
                env.config.ota.package_manager.install_package(p);
                ctx.send(Command::UpdateReport())
            }

            Event::GetInstalledSoftware => {
                env.config.ota.package_manager.installed_packages();
                ctx.send(Command::InstalledSoftware())
            }
            */

            _ => Ok(())
        }.map_err(|err| panic!("couldn't interpret event: {}", err));
    }
}


pub type Wrapped = Interpret<Command, Event>;

pub struct WrappedInterpreter<'t> {
    pub config:       Config,
    pub access_token: Option<Cow<'t, AccessToken>>,
}

impl<'t> WrappedInterpreter<'t> {
    fn interpret_command(&mut self, cmd: Command, etx: Sender<Event>) -> Result<(), Error> {
        match self.access_token.to_owned() {
            Some(token) => {
                let client = AuthClient::new(Auth::Token(token.into_owned()));
                self.authenticated(client, cmd, etx)
            }

            None => {
                let client = AuthClient::new(Auth::Credentials(
                    ClientId(self.config.auth.client_id.clone()),
                    ClientSecret(self.config.auth.secret.clone())));
                self.unauthenticated(client, cmd, etx)
            }
        }
    }

    fn authenticated(&mut self, client: AuthClient, cmd: Command, etx: Sender<Event>) -> Result<(), Error> {
        let mut ota = OTA::new(&client, self.config.clone());

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
                if updates.len() > 0 {
                    updates.sort_by_key(|update| update.installPos);
                    info!("New package updates available: {:?}", updates);
                    for update in updates.iter() {
                        try!(etx.send(Event::NewUpdateAvailable(update.requestId.clone())))
                    }
                }
            }

            ListInstalledPackages => {
                let pkgs = try!(self.config.ota.package_manager.installed_packages());
                try!(etx.send(Event::FoundInstalledPackages(pkgs)))
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

    fn unauthenticated(&mut self, client: AuthClient, cmd: Command, etx: Sender<Event>) -> Result<(), Error> {
        match cmd {
            Authenticate(_) => {
                let token = try!(authenticate(&self.config.auth, &client));
                self.access_token = Some(token.into());
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
}

impl<'t> Interpreter<Wrapped, Event> for WrappedInterpreter<'t> {
    fn interpret(&mut self, w: Wrapped, global_tx: Sender<Event>) {
        info!("Wrapped interpreter: {:?}", w.cmd);
        let (multi_tx, multi_rx): (Sender<Event>, Receiver<Event>) = channel();

        let _ = match self.interpret_command(w.cmd.clone(), multi_tx) {
            Ok(_) => {
                let mut last_ev = None;
                for ev in multi_rx {
                    last_ev = Some(ev.clone());
                    send_global(ev, global_tx.clone());
                }
                match last_ev {
                    Some(ev) => send_local(ev, w.etx),
                    None     => panic!("no local event to send back")
                }
            }

            Err(Error::AuthorizationError(_)) => {
                debug!("retry authorization and request");
                let auth = Wrapped { cmd: Command::Authenticate(None), etx: None };
                self.interpret(auth, global_tx.clone());
                self.interpret(w, global_tx);
            }

            Err(err) => {
                let ev = Event::Error(format!("{}", err));
                send_global(ev.clone(), global_tx);
                send_local(ev, w.etx);
            }
        };
    }
}

fn send_global(ev: Event, global_tx: Sender<Event>) {
    let _ = global_tx.send(ev)
                     .map_err(|err| panic!("couldn't send global response: {}", err));
}

fn send_local(ev: Event, local_tx: Option<Arc<Mutex<Sender<Event>>>>) {
    if let Some(ref local) = local_tx {
        let _ = local.lock()
                     .unwrap()
                     .send(ev)
                     .map_err(|err| panic!("couldn't send local response: {}", err));
    }
}
