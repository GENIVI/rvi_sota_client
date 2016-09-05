use chan;
use chan::{Sender, Receiver};
use std;
use std::borrow::Cow;

use datatype::{AccessToken, Auth, ClientId, ClientSecret, Command, Config,
               Error, Event, Package, UpdateRequestId};
use gateway::Interpret;
use http::{AuthClient, Client};
use oauth2::authenticate;
use package_manager::PackageManager;
use rvi::Services;
use sota::Sota;


/// An `Interpreter` loops over any incoming values, on receipt of which it
/// delegates to the `interpret` function which will respond with output values.
pub trait Interpreter<I, O> {
    fn interpret(&mut self, input: I, otx: &Sender<O>);

    fn run(&mut self, irx: Receiver<I>, otx: Sender<O>) {
        loop {
            self.interpret(irx.recv().expect("interpreter sender closed"), &otx);
        }
    }
}


/// The `EventInterpreter` listens for `Event`s and optionally responds with
/// `Command`s that may be sent to the `CommandInterpreter`.
pub struct EventInterpreter {
    pub package_manager: PackageManager
}

impl Interpreter<Event, Command> for EventInterpreter {
    fn interpret(&mut self, event: Event, ctx: &Sender<Command>) {
        info!("Event received: {}", event);
        match event {
            Event::Authenticated => {
                ctx.send(Command::SendSystemInfo);

                if self.package_manager != PackageManager::Off {
                    self.package_manager.installed_packages().map(|packages| {
                        ctx.send(Command::SendInstalledPackages(packages));
                    }).unwrap_or_else(|err| error!("couldn't send a list of packages: {}", err));
                }
            }

            Event::NotAuthenticated => {
                info!("Trying to authenticate again...");
                ctx.send(Command::Authenticate(None));
            }

            Event::NewUpdatesReceived(ids) => {
                ctx.send(Command::StartDownload(ids));
            }

            Event::DownloadComplete(dl) => {
                if self.package_manager != PackageManager::Off {
                    ctx.send(Command::StartInstall(dl));
                }
            }

            Event::InstallComplete(report) => {
                ctx.send(Command::SendUpdateReport(report));
            }

            Event::UpdateReportSent => {
                if self.package_manager != PackageManager::Off {
                    self.package_manager.installed_packages().map(|packages| {
                        ctx.send(Command::SendInstalledPackages(packages));
                    }).unwrap_or_else(|err| error!("couldn't send a list of packages: {}", err));
                }
            }

            _ => ()
        }
    }
}


/// The `CommandInterpreter` wraps each incoming `Command` inside an `Interpret`
/// type with no response channel for sending to the `GlobalInterpreter`.
pub struct CommandInterpreter;

impl Interpreter<Command, Interpret> for CommandInterpreter {
    fn interpret(&mut self, cmd: Command, itx: &Sender<Interpret>) {
        info!("Command received: {}", cmd);
        itx.send(Interpret { command: cmd, response_tx: None });
    }
}


/// The `GlobalInterpreter` interprets the `Command` inside incoming `Interpret`
/// messages, broadcasting `Event`s globally and (optionally) sending the final
/// outcome `Event` to the `Interpret` response channel.
pub struct GlobalInterpreter<'t> {
    pub config:      Config,
    pub token:       Option<Cow<'t, AccessToken>>,
    pub http_client: Box<Client>,
    pub rvi:         Option<Services>
}

impl<'t> Interpreter<Interpret, Event> for GlobalInterpreter<'t> {
    fn interpret(&mut self, interpret: Interpret, etx: &Sender<Event>) {
        info!("Interpreter started: {}", interpret.command);

        let (multi_tx, multi_rx) = chan::async::<Event>();
        let outcome = match (self.token.as_ref(), self.config.auth.is_none()) {
            (Some(_), _) | (_, true) => self.authenticated(interpret.command, multi_tx),
            _                        => self.unauthenticated(interpret.command, multi_tx)
        };

        let mut response_ev: Option<Event> = None;
        match outcome {
            Ok(_) => {
                for ev in multi_rx {
                    etx.send(ev.clone());
                    response_ev = Some(ev);
                }
                info!("Interpreter finished.");
            }

            Err(Error::Authorization(_)) => {
                let ev = Event::NotAuthenticated;
                etx.send(ev.clone());
                response_ev = Some(ev);
                error!("Interpreter authentication failed");
            }

            Err(err) => {
                let ev = Event::Error(format!("{}", err));
                etx.send(ev.clone());
                response_ev = Some(ev);
                error!("Interpreter failed: {}", err);
            }
        }

        let ev = response_ev.expect("no response event to send back");
        interpret.response_tx.map(|tx| tx.lock().unwrap().send(ev));
    }
}

impl<'t> GlobalInterpreter<'t> {
    fn authenticated(&self, cmd: Command, etx: Sender<Event>) -> Result<(), Error> {
        let mut sota = Sota::new(&self.config, self.http_client.as_ref());

        // always send at least one Event response
        match cmd {
            Command::Authenticate(_) => etx.send(Event::Authenticated),

            Command::GetNewUpdates => {
                let mut updates = try!(sota.get_pending_updates());
                if updates.is_empty() {
                    etx.send(Event::NoNewUpdates);
                } else {
                    updates.sort_by_key(|u| u.installPos);
                    let ids = updates.iter().map(|u| u.requestId.clone()).collect::<Vec<UpdateRequestId>>();
                    etx.send(Event::NewUpdatesReceived(ids))
                }
            }

            Command::ListInstalledPackages => {
                let mut packages: Vec<Package> = Vec::new();
                if self.config.device.package_manager != PackageManager::Off {
                    packages = try!(self.config.device.package_manager.installed_packages());
                }
                etx.send(Event::FoundInstalledPackages(packages));
            }

            Command::ListSystemInfo => {
                let info = try!(self.config.device.system_info.report());
                etx.send(Event::FoundSystemInfo(info));
            }

            Command::SendInstalledPackages(packages) => {
                let _ = sota.send_installed_packages(&packages)
                    .map_err(|err| error!("couldn't send installed packages: {}", err));
                etx.send(Event::InstalledPackagesSent);
            }

            Command::SendInstalledSoftware(sw) => {
                if let Some(ref rvi) = self.rvi {
                    let _ = rvi.remote.lock().unwrap().send_installed_software(sw);
                }
                etx.send(Event::InstalledSoftwareSent);
            }

            Command::SendSystemInfo => {
                let info = try!(self.config.device.system_info.report());
                let _ = sota.send_system_info(&info)
                    .map_err(|err| error!("couldn't send system info: {}", err));
                etx.send(Event::SystemInfoSent);
            }

            Command::SendUpdateReport(report) => {
                if let Some(ref rvi) = self.rvi {
                    let _ = rvi.remote.lock().unwrap().send_update_report(report);
                } else {
                    let _ = sota.send_update_report(&report)
                        .map_err(|err| error!("couldn't send update report: {}", err));
                }
                etx.send(Event::UpdateReportSent);
            }

            Command::StartDownload(ref ids) => {
                for id in ids {
                    etx.send(Event::DownloadingUpdate(id.clone()));
                    if let Some(ref rvi) = self.rvi {
                        let _ = rvi.remote.lock().unwrap().send_download_started(id.clone());
                    } else {
                        let _ = sota.download_update(id.clone())
                            .map(|dl| etx.send(Event::DownloadComplete(dl)))
                            .map_err(|err| etx.send(Event::DownloadFailed(id.clone(), format!("{}", err))));
                    }
                }
            }

            Command::StartInstall(dl) => {
                let _ = sota.install_update(dl)
                    .map(|report| etx.send(Event::InstallComplete(report)))
                    .map_err(|report| etx.send(Event::InstallFailed(report)));
            }

            Command::Shutdown => std::process::exit(0),
        }

        Ok(())
    }

    fn unauthenticated(&mut self, cmd: Command, etx: Sender<Event>) -> Result<(), Error> {
        match cmd {
            Command::Authenticate(_) => {
                let config = self.config.auth.clone().expect("trying to authenticate without auth config");
                self.set_client(Auth::Credentials(ClientId(config.client_id),
                                                  ClientSecret(config.client_secret)));
                let server = config.server.join("/token").expect("couldn't build authentication url");
                let token  = try!(authenticate(server, self.http_client.as_ref()));
                self.set_client(Auth::Token(token.clone()));
                self.token = Some(token.into());
                etx.send(Event::Authenticated);
            }

            Command::GetNewUpdates            |
            Command::ListInstalledPackages    |
            Command::ListSystemInfo           |
            Command::SendInstalledPackages(_) |
            Command::SendInstalledSoftware(_) |
            Command::SendUpdateReport(_)      |
            Command::SendSystemInfo           |
            Command::StartDownload(_)         |
            Command::StartInstall(_)          => etx.send(Event::NotAuthenticated),

            Command::Shutdown => std::process::exit(0),
        }

        Ok(())
    }

    fn set_client(&mut self, auth: Auth) {
        if !self.http_client.is_testing() {
            self.http_client = Box::new(AuthClient::from(auth));
        }
    }
}


#[cfg(test)]
mod tests {
    use chan;
    use chan::{Sender, Receiver};
    use std::thread;

    use super::*;
    use datatype::{AccessToken, Command, Config, DownloadComplete, Event,
                   UpdateReport, UpdateResultCode};
    use gateway::Interpret;
    use http::test_client::TestClient;
    use package_manager::PackageManager;
    use package_manager::tpm::assert_rx;


    fn new_interpreter(replies: Vec<String>, pkg_mgr: PackageManager) -> (Sender<Command>, Receiver<Event>) {
        let (etx, erx) = chan::sync::<Event>(0);
        let (ctx, crx) = chan::sync::<Command>(0);

        thread::spawn(move || {
            let mut gi = GlobalInterpreter {
                config:      Config::default(),
                token:       Some(AccessToken::default().into()),
                http_client: Box::new(TestClient::from(replies)),
                rvi:         None
            };
            gi.config.device.package_manager = pkg_mgr;

            loop {
                match crx.recv() {
                    Some(cmd) => gi.interpret(Interpret { command: cmd, response_tx: None }, &etx),
                    None      => break
                }
            }
        });

        (ctx, erx)
    }

    #[test]
    fn already_authenticated() {
        let replies    = Vec::new();
        let pkg_mgr    = PackageManager::new_tpm(true);
        let (ctx, erx) = new_interpreter(replies, pkg_mgr);

        ctx.send(Command::Authenticate(None));
        assert_rx(erx, &[Event::Authenticated]);
    }

    #[test]
    fn download_updates() {
        let replies    = vec!["[]".to_string(); 10];
        let pkg_mgr    = PackageManager::new_tpm(true);
        let (ctx, erx) = new_interpreter(replies, pkg_mgr);

        ctx.send(Command::StartDownload(vec!["1".to_string(), "2".to_string()]));
        assert_rx(erx, &[
            Event::DownloadingUpdate("1".to_string()),
            Event::DownloadComplete(DownloadComplete {
                update_id:    "1".to_string(),
                update_image: "/tmp/1".to_string(),
                signature:    "".to_string()
            }),
            Event::DownloadingUpdate("2".to_string()),
            Event::DownloadComplete(DownloadComplete {
                update_id:    "2".to_string(),
                update_image: "/tmp/2".to_string(),
                signature:    "".to_string()
            }),
        ]);
    }

    #[test]
    fn install_update_success() {
        let replies    = vec!["[]".to_string(); 10];
        let pkg_mgr    = PackageManager::new_tpm(true);
        let (ctx, erx) = new_interpreter(replies, pkg_mgr);

        ctx.send(Command::StartInstall(DownloadComplete {
            update_id:    "1".to_string(),
            update_image: "/tmp/1".to_string(),
            signature:    "".to_string()
        }));
        assert_rx(erx, &[
            Event::InstallComplete(
                UpdateReport::single("1".to_string(), UpdateResultCode::OK, "".to_string())
            )
        ]);
    }

    #[test]
    fn install_update_failed() {
        let replies    = vec!["[]".to_string(); 10];
        let pkg_mgr    = PackageManager::new_tpm(false);
        let (ctx, erx) = new_interpreter(replies, pkg_mgr);

        ctx.send(Command::StartInstall(DownloadComplete {
            update_id:    "1".to_string(),
            update_image: "/tmp/1".to_string(),
            signature:    "".to_string()
        }));
        assert_rx(erx, &[
            Event::InstallFailed(
                UpdateReport::single("1".to_string(), UpdateResultCode::INSTALL_FAILED, "failed".to_string())
            ),
        ]);
    }
}
