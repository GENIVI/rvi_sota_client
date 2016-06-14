use std::borrow::Cow;
use std::process::exit;
use std::sync::mpsc::{Sender, Receiver, channel};

use datatype::{AccessToken, Auth, ClientId, ClientSecret, Command, Config,
               Error, Event, UpdateState, UpdateRequestId};
use datatype::Command::*;
use http_client::{AuthClient, HttpClient};
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


pub struct CommandInterpreter;

impl Interpreter<Command, Wrapped> for CommandInterpreter {
    fn interpret(&mut self, cmd: Command, wtx: &Sender<Wrapped>) {
        info!("Command interpreter: {:?}", cmd);
        let _ = wtx.send(Wrapped { cmd: cmd, etx: None })
                   .map_err(|err| panic!("couldn't forward command: {}", err));
    }
}


pub type Wrapped = Interpret<Command, Event>;

impl Wrapped {
    fn publish(&self, ev: Event) {
        if let Some(ref etx) = self.etx {
            let _ = etx.lock().unwrap().send(ev).map_err(|err| panic!("couldn't publish event: {}", err));
        }
    }
}


pub struct WrappedInterpreter<'t> {
    pub config:   Config,
    pub token:    Option<Cow<'t, AccessToken>>,
    pub client:   Box<HttpClient>,
    pub loopback: Sender<Wrapped>,
}

impl<'t> WrappedInterpreter<'t> {
    fn authenticated(&self, cmd: Command, etx: Sender<Event>) -> Result<(), Error> {
        let mut ota = OTA::new(&self.config, self.client.as_ref());

        // always send at least one Event response
        match cmd {
            AcceptUpdates(ids) => {
                for id in ids {
                    info!("Accepting id {}", id);
                    try!(etx.send(Event::UpdateStateChanged(id.clone(), UpdateState::Downloading)));
                    let report = try!(ota.install_package_update(&id, &etx));
                    try!(ota.send_install_report(&report));
                    info!("Install Report for {}: {:?}", id, report);
                    try!(ota.update_installed_packages())
                }
            }

            Authenticate(_) => try!(etx.send(Event::Ok)),

            GetPendingUpdates => {
                let mut updates = try!(ota.get_package_updates());
                if updates.len() > 0 {
                    updates.sort_by_key(|u| u.installPos);
                    info!("New package updates available: {:?}", updates);
                    let ids: Vec<UpdateRequestId> = updates.iter().map(|u| u.requestId.clone()).collect();
                    let w = Wrapped { cmd: Command::AcceptUpdates(ids), etx: None };
                    try!(self.loopback.send(w))
                }
                try!(etx.send(Event::Ok));
            }

            ListInstalledPackages => {
                let pkgs = try!(self.config.ota.package_manager.installed_packages());
                try!(etx.send(Event::FoundInstalledPackages(pkgs)))
            }

            Shutdown => exit(0),

            UpdateInstalledPackages => {
                try!(ota.update_installed_packages());
                try!(etx.send(Event::Ok));
                info!("Posted installed packages to the server.")
            }
        }

        Ok(())
    }

    fn unauthenticated(&mut self, cmd: Command, etx: Sender<Event>) -> Result<(), Error> {
        match cmd {
            Authenticate(_) => {
                let token = try!(authenticate(&self.config.auth, self.client.as_ref()));
                self.token = Some(token.into());
                try!(etx.send(Event::Authenticated));
            }

            AcceptUpdates(_)      |
            GetPendingUpdates     |
            ListInstalledPackages |
            UpdateInstalledPackages => try!(etx.send(Event::NotAuthenticated)),

            Shutdown => exit(0),
        }

        Ok(())
    }
}

impl<'t> Interpreter<Wrapped, Event> for WrappedInterpreter<'t> {
    fn interpret(&mut self, w: Wrapped, global_tx: &Sender<Event>) {
        info!("Wrapped interpreter: {:?}", w.cmd);
        let broadcast = |ev: Event| {
            let _ = global_tx.send(ev).map_err(|err| panic!("couldn't broadcast event: {}", err));
        };

        let (etx, erx): (Sender<Event>, Receiver<Event>) = channel();
        let outcome = match self.token.to_owned() {
            Some(token) => {
                if !self.client.is_testing() {
                    self.client = Box::new(AuthClient::new(Auth::Token(token.into_owned())));
                }
                self.authenticated(w.cmd.clone(), etx)
            }

            None => {
                if !self.client.is_testing() {
                    self.client = Box::new(AuthClient::new(Auth::Credentials(
                        ClientId(self.config.auth.client_id.clone()),
                        ClientSecret(self.config.auth.secret.clone()))));
                }
                self.unauthenticated(w.cmd.clone(), etx)
            }
        };

        match outcome {
            Ok(_) => {
                let mut last_ev = None;
                for ev in erx {
                    broadcast(ev.clone());
                    last_ev = Some(ev);
                }
                match last_ev {
                    Some(ev) => w.publish(ev),
                    None     => panic!("no local event to send back")
                };
            }

            Err(Error::AuthorizationError(_)) => {
                debug!("retry authorization and request");
                let a = Wrapped { cmd: Command::Authenticate(None), etx: None };
                let _ = self.loopback.send(a).map_err(|err| panic!("couldn't retry authentication: {}", err));
                let _ = self.loopback.send(w).map_err(|err| panic!("couldn't retry request: {}", err));
            }

            Err(err) => {
                let ev = Event::Error(format!("{}", err));
                broadcast(ev.clone());
                w.publish(ev);
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use std::thread;
    use std::sync::mpsc::{channel, Sender, Receiver};

    use super::*;
    use datatype::{AccessToken, Command, Config, Event, UpdateState};
    use http_client::test_client::TestHttpClient;
    use package_manager::PackageManager;
    use package_manager::tpm::assert_rx;


    fn new_interpreter(replies: Vec<String>, pkg_mgr: PackageManager) -> (Sender<Command>, Receiver<Event>) {
        let (ctx, crx): (Sender<Command>, Receiver<Command>) = channel();
        let (etx, erx): (Sender<Event>,   Receiver<Event>)   = channel();
        let (wtx, _):   (Sender<Wrapped>, Receiver<Wrapped>) = channel();

        thread::spawn(move || {
            let mut wi = WrappedInterpreter {
                config:   Config::default(),
                token:    Some(AccessToken::default().into()),
                client:   Box::new(TestHttpClient::from(replies)),
                loopback: wtx,
            };
            wi.config.ota.package_manager = pkg_mgr;
            loop {
                match crx.recv().expect("couldn't receive cmd") {
                    Command::Shutdown => break,
                    cmd @ _ => wi.interpret(Wrapped { cmd: cmd, etx: None }, &etx)
                }
            }
        });

        (ctx, erx)
    }

    #[test]
    fn already_authenticated() {
        let (ctx, erx) = new_interpreter(Vec::new(), PackageManager::new_file(true));
        ctx.send(Command::Authenticate(None)).unwrap();
        for ev in erx.recv() {
            assert_eq!(ev, Event::Ok);
        }
        ctx.send(Command::Shutdown).unwrap();
    }

    #[test]
    fn accept_updates() {
        let replies    = vec!["[]".to_string(); 10];
        let (ctx, erx) = new_interpreter(replies, PackageManager::new_file(true));

        ctx.send(Command::AcceptUpdates(vec!["1".to_string(), "2".to_string()])).unwrap();
        assert_rx(erx, &[
            Event::UpdateStateChanged("1".to_string(), UpdateState::Downloading),
            Event::UpdateStateChanged("1".to_string(), UpdateState::Installing),
            Event::UpdateStateChanged("1".to_string(), UpdateState::Installed),
            Event::UpdateStateChanged("2".to_string(), UpdateState::Downloading),
            Event::UpdateStateChanged("2".to_string(), UpdateState::Installing),
            Event::UpdateStateChanged("2".to_string(), UpdateState::Installed),
        ]);
        ctx.send(Command::Shutdown).unwrap();
    }

    #[test]
    fn failed_updates() {
        let replies    = vec!["[]".to_string(); 10];
        let pkg_mgr    = PackageManager::new_file(false);
        let (ctx, erx) = new_interpreter(replies, pkg_mgr);

        ctx.send(Command::AcceptUpdates(vec!["1".to_string()])).unwrap();
        assert_rx(erx, &[
            Event::Error("IO error: No such file or directory (os error 2)".to_owned()),
        ]);
        ctx.send(Command::Shutdown).unwrap();
    }
}
