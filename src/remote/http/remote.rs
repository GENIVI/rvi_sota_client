use std::sync::mpsc::Sender;
use remote::upstream::Upstream;
use configuration::ServerConfiguration;
use event::outbound::{InstalledSoftware, UpdateReport};
use event::inbound::{DownloadComplete, InboundEvent};
use event::Event;
use event::UpdateId;

use super::HttpClient;
use super::api_client::{update_packages, download_package_update, send_install_report};
use super::datatype::AccessToken;

pub struct HttpRemote<C: HttpClient> {
    config: ServerConfiguration,
    access_token: Option<AccessToken>,
    client: C,
    tx: Sender<Event>
}

impl<C: HttpClient> HttpRemote<C> {
    pub fn new(config: ServerConfiguration, access_token: Option<AccessToken>, client: C, tx: Sender<Event>) -> HttpRemote<C> {
        HttpRemote { config: config, access_token: access_token, client: client, tx: tx }
    }
}

impl<C: HttpClient> Upstream for HttpRemote<C> {
    fn send_installed_software(&mut self, m: InstalledSoftware) -> Result<String, String> {
        update_packages(&self.config, self.access_token.clone(), &mut self.client, &m.packages)
            .map(|_| "ok".to_string())
            .map_err(|e| format!("{}", e))
    }

    fn send_start_download(&mut self, id: UpdateId) -> Result<String, String> {
        download_package_update(&self.config, self.access_token.clone(), &mut self.client, &id)
            .map_err(|e| format!("{}", e))
            .and_then(|p| {
                let path = p.to_str().unwrap().to_string();
                let event = Event::Inbound(InboundEvent::DownloadComplete(DownloadComplete {
                    update_id: id,
                    update_image: path,
                    signature: "signature".to_string()
                }));
                self.tx.send(event).map(|_| "ok".to_string()).map_err(|_| "send error".to_string())
            })
    }

    fn send_update_report(&mut self, m: UpdateReport) -> Result<String, String> {
        send_install_report(&self.config, self.access_token.clone(), &mut self.client, &m)
            .map(|_| "ok".to_string())
            .map_err(|e| format!("{}", e))
    }
}