use std::sync::mpsc::{Sender , Receiver};
use std::marker::PhantomData;
use std::process::exit;

use http_client::HttpClient;
use ota_plus::{get_package_updates, download_package_update, post_packages, send_install_report};
use datatype::{Event, Command, Config, AccessToken, UpdateState, Package, Error, UpdateRequestId, UpdateReport, UpdateResultCode};

pub struct Interpreter<'a, C: HttpClient> {
    client_type: PhantomData<C>,
    config: &'a Config,
    token: AccessToken,
    // Commands mpsc, events spmc
    commands_rx: Receiver<Command>,
    events_tx: Sender<Event>
}

impl<'a, C: HttpClient> Interpreter<'a, C> {
    pub fn new(config: &'a Config, token: AccessToken, commands_rx: Receiver<Command>, events_tx: Sender<Event>) -> Interpreter<'a, C> {
        Interpreter { client_type: PhantomData, config: config, token: token, commands_rx: commands_rx, events_tx: events_tx }
    }

    pub fn start(&self) {
        loop {
            match self.commands_rx.recv() {
                Ok(cmd) => self.interpret(cmd),
                Err(e) => error!("Error receiving command: {:?}", e)
            }
        }
    }

    pub fn interpret(&self, command: Command) {
        match command {
            Command::GetPendingUpdates => self.get_pending_updates(),
            Command::PostInstalledPackages => self.post_installed_packages(),
            Command::AcceptUpdate(ref id) => self.accept_update(id),
            Command::ListInstalledPackages => self.list_installed_packages(),
            Command::Shutdown => {
                info!("Shutting down...");
                exit(0)
            }
        }
    }

    fn publish(&self, event: Event) {
        let _ = self.events_tx.send(event);
    }

    fn get_installed_packages(&self) -> Result<Vec<Package>, Error> {
        self.config.ota.package_manager.installed_packages()
    }

    fn get_pending_updates(&self) {
        debug!("Fetching package updates...");
        let response: Event = match get_package_updates::<C>(&self.token, &self.config) {
            Ok(updates) => {
                let update_events: Vec<Event> = updates.iter().map(move |id| Event::NewUpdateAvailable(id.clone())).collect();
                info!("New package updates available: {:?}", update_events);
                Event::Batch(update_events)
            },
            Err(e) => {
                Event::Error(format!("{}", e))
            }
        };
        self.publish(response);
    }

    fn post_installed_packages(&self) {
        let _ = self.get_installed_packages().and_then(|pkgs| {
            debug!("Found installed packages in the system: {:?}", pkgs);
            post_packages::<C>(&self.token, &self.config, &pkgs)
        }).map(|_| {
            info!("Posted installed packages to the server.");
        }).map_err(|e| {
            error!("Error fetching/posting installed packages: {:?}.", e);
        });
    }

    fn accept_update(&self, id: &UpdateRequestId) {
        info!("Accepting update {}...", id);
        self.publish(Event::UpdateStateChanged(id.clone(), UpdateState::Downloading));
        let report = download_package_update::<C>(&self.token, &self.config, id)
            .and_then(|path| {
                info!("Downloaded at {:?}. Installing...", path);
                self.publish(Event::UpdateStateChanged(id.clone(), UpdateState::Installing));

                let p = try!(path.to_str().ok_or(Error::ParseError(format!("Path is not valid UTF-8: {:?}", path))));
                self.config.ota.package_manager.install_package(p)
                    .map(|(code, output)| {
                        self.publish(Event::UpdateStateChanged(id.clone(), UpdateState::Installed));
                        self.post_installed_packages();
                        UpdateReport::new(id.clone(), code, output)
                    })
                    .or_else(|(code, output)| {
                        self.publish(Event::UpdateErrored(id.clone(), format!("{:?}: {:?}", code, output)));
                        Ok(UpdateReport::new(id.clone(), code, output))
                    })
            }).unwrap_or_else(|e| {
                self.publish(Event::UpdateErrored(id.clone(), format!("{:?}", e)));
                UpdateReport::new(id.clone(),
                                   UpdateResultCode::GENERAL_ERROR,
                                   format!("Download failed: {:?}", e))
            });

        match send_install_report::<C>(&self.token, &self.config, &report) {
            Ok(_) => info!("Update finished. Report sent: {:?}", report),
            Err(e) => error!("Error reporting back to the server: {:?}", e)
        }
    }

    fn list_installed_packages(&self) {
        let _ = self.get_installed_packages().and_then(|pkgs| {
            self.publish(Event::FoundInstalledPackages(pkgs.clone()));
            Ok(())
        });
    }
}
