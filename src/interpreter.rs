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
    fn interpret(&mut self, msg: I, otx: &Sender<O>);

    fn run(&mut self, irx: Receiver<I>, otx: Sender<O>) {
        loop {
            let _ = irx.recv()
                       .map(|msg| self.interpret(msg, &otx))
                       .map_err(|err| panic!("couldn't read interpreter input: {:?}", err));
        }
    }
}


pub struct EventInterpreter;

impl Interpreter<Event, Command> for EventInterpreter {
    fn interpret(&mut self, event: Event, ctx: &Sender<Command>) {
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
    pub wtx:          Sender<Wrapped>,
}

impl<'t> Interpreter<Wrapped, Event> for WrappedInterpreter<'t> {
    fn interpret(&mut self, w: Wrapped, global_tx: &Sender<Event>) {
        fn send_global(ev: Event, global_tx: &Sender<Event>) {
            let _ = global_tx.send(ev).map_err(|err| panic!("couldn't send global response: {}", err));
        }

        fn send_local(ev: Event, local_tx: Option<Arc<Mutex<Sender<Event>>>>) {
            if let Some(local) = local_tx {
                let _ = local.lock().unwrap().send(ev)
                    .map_err(|err| panic!("couldn't send local response: {}", err));
            }
        }

        info!("Interpreting wrapped command: {:?}", w.cmd);
        let (multi_tx, multi_rx): (Sender<Event>, Receiver<Event>) = channel();
        match match self.access_token.to_owned() {
            Some(token) => self.authenticated(w.cmd.clone(), token.into_owned(), multi_tx),
            None        => self.unauthenticated(w.cmd.clone(), multi_tx)
        }{
            Ok(_) => {
                let mut last_ev = None;
                for ev in multi_rx {
                    send_global(ev.clone(), &global_tx);
                    last_ev = Some(ev);
                }
                match last_ev {
                    Some(ev) => send_local(ev, w.etx),
                    None     => panic!("no local event to send back")
                };
            }

            Err(Error::AuthorizationError(_)) => {
                debug!("retry authorization and request");
                let a = Wrapped { cmd: Command::Authenticate(None), etx: None };
                let _ = self.wtx.send(a).map_err(|err| panic!("couldn't retry authentication: {}", err));
                let _ = self.wtx.send(w).map_err(|err| panic!("couldn't retry request: {}", err));
            }

            Err(err) => {
                let ev = Event::Error(format!("{}", err));
                send_global(ev.clone(), &global_tx);
                send_local(ev, w.etx);
            }
        }
    }
}


impl<'t> WrappedInterpreter<'t> {
    fn authenticated(&self, cmd: Command, token: AccessToken, etx: Sender<Event>) -> Result<(), Error> {
        let client  = AuthClient::new(Auth::Token(token));
        let mut ota = OTA::new(&self.config, &client);

        // always send at least one Event response
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
                if updates.len() == 0 {
                    return Ok(try!(etx.send(Event::Ok)));
                }
                updates.sort_by_key(|update| update.installPos);
                info!("New package updates available: {:?}", updates);
                for update in updates.iter() {
                    try!(etx.send(Event::NewUpdateAvailable(update.requestId.clone())))
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

    fn unauthenticated(&mut self, cmd: Command, etx: Sender<Event>) -> Result<(), Error> {
        match cmd {
            Authenticate(_) => {
                let client = AuthClient::new(Auth::Credentials(
                    ClientId(self.config.auth.client_id.clone()),
                    ClientSecret(self.config.auth.secret.clone())));
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
