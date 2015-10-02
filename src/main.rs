// TODO: proper argument parsing
//       See crates docopt or getopts
// TODO: Split main loop and setup into its own module
extern crate sota_client;
#[macro_use] extern crate log;
extern crate env_logger;

use sota_client::rvi;
use sota_client::handler::ServiceHandler;
use sota_client::message::{InitiateParams, BackendServices};
use sota_client::message::Notification;
use sota_client::configuration::Configuration;
use sota_client::sota_dbus;

use std::env;
use std::sync::mpsc::channel;
use std::thread;

/// Start a SOTA client service listenenig on the provided address/port combinations
#[cfg_attr(test, allow(dead_code))]
fn main() {
    env_logger::init().unwrap();

    let conf_file = Configuration::default_path();
    let configuration = match Configuration::read(&conf_file) {
        Ok(value) => value,
        Err(e) => {
            error!("Couldn't parse configuration file at {}: {}", conf_file, e);
            std::process::exit(126);
        }
    };

    let mut args = env::args();
    args.next();

    let rvi_url: String = args.next().unwrap_or(
        configuration.client.rvi_url.unwrap_or(
        "http://localhost:8901".to_string()));
    let edge_url: String = args.next().unwrap_or(
        configuration.client.edge_url.unwrap_or(
        "localhost:18901".to_string()));

    // will receive RVI registration details
    let (tx_edge, rx_edge) = channel();
    let rvi_edge = rvi::ServiceEdge::new(rvi_url.clone(),
                                         edge_url.clone(),
                                         tx_edge);

    // will receive notifies from RVI and install requests from dbus
    let (tx_main, rx_main) = channel();
    let handler = ServiceHandler::new(tx_main.clone(),
                                      rvi_url.clone(),
                                      configuration.client.storage_dir.clone());

    let services = vec!["/sota/notify",
                        "/sota/start",
                        "/sota/chunk",
                        "/sota/finish"];

    thread::spawn(move || {
        rvi_edge.start(handler, services);
    });

    let (tx_dbus, rx_dbus) = channel();
    let dbus_sender = sota_dbus::Sender::new(configuration.dbus.clone(),
                                             rx_dbus, tx_main.clone());
    thread::spawn(move || {
        dbus_sender.start();
    });

    let dbus_receiver = sota_dbus::Receiver::new(configuration.dbus,
                                                 tx_main.clone());
    thread::spawn(move || {
        dbus_receiver.start();
    });

    let local_services = rx_edge.recv().unwrap();
    let mut backend_services = BackendServices::new();

    loop {
        match rx_main.recv().unwrap() {
            Notification::Notify(notify) => {
                backend_services.update(&notify.services);
                let message = sota_dbus::Request::Notify(notify.packages);
                let _ = tx_dbus.send(message);
            },
            Notification::Initiate(packages) => {
                let initiate = InitiateParams::new(packages, &local_services);
                match rvi::send_message(&rvi_url, initiate,
                                        &backend_services.start) {
                    Ok(..) => {},
                    Err(e) => { error!("Couldn't initiate download: {}", e); }
                }
            }
            Notification::Finish(package) => {
                tx_dbus.send(sota_dbus::Request::Complete(package)).unwrap();
            },
            Notification::InstallReport(report) => {
                trace!("Got installation report: {:?}", report);
                match rvi::send_message(&rvi_url, report,
                                        &backend_services.report) {
                    Ok(..) => {},
                    Err(e) => { error!("Couldn't send install report: {}", e); }
                }
            },
            Notification::Report(_) => {} // TODO: SOTA-129
        }
    }
}
