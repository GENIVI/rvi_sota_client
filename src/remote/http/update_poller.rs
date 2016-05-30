use super::api_client::get_package_updates;

use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

use configuration::ServerConfiguration;

use event::inbound::InboundEvent;
use event::Event;

use super::HttpClient;
use super::hyper::Hyper;
use super::AccessToken;

pub fn start(config: ServerConfiguration,
             access_token: Option<AccessToken>,
             tx: Sender<Event>) {

    thread::spawn(move || {
        let mut c: &mut HttpClient = &mut Hyper::new();
        let t = access_token;
        loop {
            match get_package_updates(&config, t.clone(), c) {
                Ok(updates) =>
                    for update in updates {
                        let _ = tx.send(Event::Inbound(InboundEvent::UpdateAvailable(update)));
                    },
                Err(e) => error!("Can't get package updates: {:?}", e)
            }
            thread::sleep(Duration::from_secs(config.polling_interval as u64));
        }
    });
}
