use chan;
use chan::{Sender, Receiver};
use std;
use std::borrow::Cow;

use datatype::{AccessToken, Auth, ClientId, ClientSecret, Command, Config,
               Error, Event, UpdateState, UpdateRequestId};
use datatype::Command::*;
use http_client::{AuthClient, HttpClient};
use interaction_library::gateway::Interpret;
use oauth2::authenticate;
use ota_plus::OTA;


pub trait Interpreter<I: 'static, O> {
    fn interpret(&mut self, msg: I, otx: &Sender<O>);

    fn run(&mut self, irx: Receiver<I>, otx: Sender<O>) {
        loop {
            match irx.recv() {
                Some(i) => self.interpret(i, &otx),
                None    => panic!("interpreter sender closed")
            }
        }
    }
}


pub struct EventInterpreter;

impl Interpreter<Event, Command> for EventInterpreter {
    fn interpret(&mut self, event: Event, ctx: &Sender<Command>) {
        info!("Event interpreter: {:?}", event);
        match event {
            Event::FoundInstalledPackages(pkgs) => {
                info!("Installed packages: {:?}", pkgs);
            }

            Event::NotAuthenticated => {
                debug!("Trying to authenticate again...");
                ctx.send(Command::Authenticate(None));
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

            _ => ()
        }
    }
}


pub struct CommandInterpreter;

impl Interpreter<Command, Global> for CommandInterpreter {
    fn interpret(&mut self, cmd: Command, gtx: &Sender<Global>) {
        info!("Command interpreter: {:?}", cmd);
        gtx.send(Global { command: cmd, response_tx: None });
    }
}


pub type Global = Interpret<Command, Event>;

pub struct GlobalInterpreter<'t> {
    pub config:      Config,
    pub token:       Option<Cow<'t, AccessToken>>,
    pub http_client: Box<HttpClient>,
    pub loopback_tx: Sender<Global>,
}

impl<'t> Interpreter<Global, Event> for GlobalInterpreter<'t> {
    fn interpret(&mut self, global: Global, etx: &Sender<Event>) {
        info!("Global interpreter started: {:?}", global.command);
        let response = |ev: Event| {
            if let Some(ref response_tx) = global.response_tx {
                response_tx.lock().unwrap().send(ev)
            }
        };

        let (multi_tx, multi_rx) = chan::async::<Event>();
        let outcome = match self.token {
            Some(_) => self.authenticated(global.command.clone(), multi_tx),
            None    => self.unauthenticated(global.command.clone(), multi_tx)
        };

        let mut response_ev: Option<Event> = None;
        match outcome {
            Ok(_) => {
                for ev in multi_rx {
                    etx.send(ev.clone());
                    response_ev = Some(ev);
                }
                info!("Global interpreter success: {:?}", global.command);
            }

            Err(Error::AuthorizationError(_)) => {
                let ev = Event::NotAuthenticated;
                etx.send(ev.clone());
                response_ev = Some(ev);
                error!("Global interpreter authentication failed: {:?}", global.command);
            }

            Err(err) => {
                let ev = Event::Error(format!("{}", err));
                etx.send(ev.clone());
                response_ev = Some(ev);
                error!("Global interpreter failed: {:?}: {}", global.command, err);
            }
        }

        match response_ev {
            Some(ev) => response(ev),
            None     => panic!("no response event to send back")
        };
    }
}

impl<'t> GlobalInterpreter<'t> {
    fn authenticated(&self, cmd: Command, etx: Sender<Event>) -> Result<(), Error> {
        let mut ota = OTA::new(&self.config, self.http_client.as_ref());

        // always send at least one Event response
        match cmd {
            AcceptUpdates(ids) => {
                for id in ids {
                    info!("Accepting ID: {}", id);
                    etx.send(Event::UpdateStateChanged(id.clone(), UpdateState::Downloading));
                    let report = try!(ota.install_package_update(&id, &etx));
                    try!(ota.send_install_report(&report));
                    info!("Install Report for {}: {:?}", id, report);
                    try!(ota.update_installed_packages())
                }
            }

            Authenticate(_) => etx.send(Event::Ok),

            GetPendingUpdates => {
                let mut updates = try!(ota.get_package_updates());
                if updates.len() > 0 {
                    updates.sort_by_key(|u| u.installPos);
                    info!("New package updates available: {:?}", updates);
                    let ids: Vec<UpdateRequestId> = updates.iter().map(|u| u.requestId.clone()).collect();
                    self.loopback_tx.send(Global { command: Command::AcceptUpdates(ids), response_tx: None });
                }
                etx.send(Event::Ok);
            }

            ListInstalledPackages => {
                let pkgs = try!(self.config.ota.package_manager.installed_packages());
                etx.send(Event::FoundInstalledPackages(pkgs));
            }

            Shutdown => std::process::exit(0),

            UpdateInstalledPackages => {
                try!(ota.update_installed_packages());
                etx.send(Event::Ok);
                info!("Posted installed packages to the server.")
            }
        }

        Ok(())
    }

    fn unauthenticated(&mut self, cmd: Command, etx: Sender<Event>) -> Result<(), Error> {
        match cmd {
            Authenticate(_) => {
                let auth = Auth::Credentials(ClientId(self.config.auth.client_id.clone()),
                                             ClientSecret(self.config.auth.secret.clone()));
                self.set_client(auth);
                let token = try!(authenticate(&self.config.auth, self.http_client.as_ref()));
                self.set_client(Auth::Token(token.clone()));
                self.token = Some(token.into());
                etx.send(Event::Authenticated);
            }

            AcceptUpdates(_)      |
            GetPendingUpdates     |
            ListInstalledPackages |
            UpdateInstalledPackages => etx.send(Event::NotAuthenticated),

            Shutdown => std::process::exit(0),
        }

        Ok(())
    }

    fn set_client(&mut self, auth: Auth) {
        if !self.http_client.is_testing() {
            self.http_client = Box::new(AuthClient::new(auth));
        }
    }
}


#[cfg(test)]
mod tests {
    use chan;
    use chan::{Sender, Receiver};
    use std::thread;

    use super::*;
    use datatype::{AccessToken, Command, Config, Event, UpdateState};
    use http_client::test_client::TestHttpClient;
    use package_manager::PackageManager;
    use package_manager::tpm::assert_rx;


    fn new_interpreter(replies: Vec<String>, pkg_mgr: PackageManager) -> (Sender<Command>, Receiver<Event>) {
        let (etx, erx) = chan::sync::<Event>(0);
        let (ctx, crx) = chan::sync::<Command>(0);
        let (gtx, _)   = chan::sync::<Global>(0);

        thread::spawn(move || {
            let mut wi = GlobalInterpreter {
                config:      Config::default(),
                token:       Some(AccessToken::default().into()),
                http_client: Box::new(TestHttpClient::from(replies)),
                loopback_tx: gtx,
            };
            wi.config.ota.package_manager = pkg_mgr;

            loop {
                match crx.recv() {
                    Some(cmd) => wi.interpret(Global { command: cmd, response_tx: None }, &etx),
                    None      => break
                }
            }
        });

        (ctx, erx)
    }

    #[test]
    fn already_authenticated() {
        let replies    = Vec::new();
        let pkg_mgr    = PackageManager::new_file(true);
        let (ctx, erx) = new_interpreter(replies, pkg_mgr);

        ctx.send(Command::Authenticate(None));
        assert_rx(erx, &[Event::Ok]);
    }

    #[test]
    fn accept_updates() {
        let replies    = vec!["[]".to_string(); 10];
        let pkg_mgr    = PackageManager::new_file(true);
        let (ctx, erx) = new_interpreter(replies, pkg_mgr);

        ctx.send(Command::AcceptUpdates(vec!["1".to_string(), "2".to_string()]));
        assert_rx(erx, &[
            Event::UpdateStateChanged("1".to_string(), UpdateState::Downloading),
            Event::UpdateStateChanged("1".to_string(), UpdateState::Installing),
            Event::UpdateStateChanged("1".to_string(), UpdateState::Installed),
            Event::UpdateStateChanged("2".to_string(), UpdateState::Downloading),
            Event::UpdateStateChanged("2".to_string(), UpdateState::Installing),
            Event::UpdateStateChanged("2".to_string(), UpdateState::Installed),
        ]);
    }

    #[test]
    fn failed_updates() {
        let replies    = vec!["[]".to_string(); 10];
        let pkg_mgr    = PackageManager::new_file(false);
        let (ctx, erx) = new_interpreter(replies, pkg_mgr);

        ctx.send(Command::AcceptUpdates(vec!["1".to_string()]));
        assert_rx(erx, &[Event::Error("IO error: No such file or directory (os error 2)".to_owned())]);
    }
}
