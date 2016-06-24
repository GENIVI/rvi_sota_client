use std::sync::{Arc, Mutex};
use chan::Sender;

use datatype::{Command, Event};
use datatype::Command::*;

use interpreter::Interpreter;
use interaction_library::gateway::Interpret;

pub struct SwmEventInterpreter;

use datatype::config::DBusConfiguration;
use swm::swlm;

impl Interpreter<Event, Command> for SwmEventInterpreter {
    fn interpret(&mut self, event: Event, ctx: &Sender<Command>) {
        let cfg = DBusConfiguration::default();
        info!("Event interpreter: {:?}", event);
        match event {
            Event::UpdateAvailable(e) => {
                swlm::send_update_available(&cfg, e);
            }
            Event::DownloadComplete(e) => {
                swlm::send_download_complete(&cfg, e);
            }
            Event::GetInstalledSoftware(e) => {
                let _ = swlm::send_get_installed_software(&cfg, e)
                    .map(|e| ctx.send(Command::ReportInstalledSoftware(e)));
            }
            _ => {}
        }
    }
}


use remote::upstream::Upstream;
pub struct UpstreamInterpreter<U> where U: Upstream {
    pub upstream: Arc<Mutex<U>>
}

pub type Global = Interpret<Command, Event>;

impl<U> Interpreter<Global, Event> for UpstreamInterpreter<U> where U: Upstream{
    fn interpret(&mut self, w: Global, _global_tx: &Sender<Event>) {

        info!("UpstreamInterpreter: {:?}", w.command);
        match w.command {
            AcceptUpdates(ids) => {
                let _ = self.upstream.lock().unwrap().send_start_download(ids[0].clone());
            }

            UpdateReport(report) => {
                let _ = self.upstream.lock().unwrap().send_update_report(report);
            }

            ReportInstalledSoftware(installed) => {
                let _ = self.upstream.lock().unwrap().send_installed_software(installed);
            }

            _ => {}
        }
    }
}
