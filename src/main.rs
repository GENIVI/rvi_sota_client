// TODO: rather use custom types instead of primitives, to get more type safety
// TODO: proper argument parsing
extern crate sota_client;
#[macro_use] extern crate log;
extern crate env_logger;

use sota_client::rvi;
use sota_client::handler::ServiceHandler;
use sota_client::message::InitiateParams;
use sota_client::configuration::Configuration;

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

    let (tx_edge, rx_edge) = channel();
    let rvi_edge = rvi::ServiceEdge::new(rvi_url.clone(),
                                         edge_url.clone(),
                                         tx_edge);

    let (tx_handler, rx_handler) = channel();
    let handler = ServiceHandler::new(tx_handler,
                                      rvi_url.clone(),
                                      &configuration.client);

    let services = vec!["/sota/notify",
                        "/sota/start",
                        "/sota/chunk",
                        "/sota/finish"];

    thread::spawn(move || {
        rvi_edge.start(handler, services);
    });

    let services = rx_edge.recv().unwrap();

    loop {
        let message = rx_handler.recv().unwrap();
        // In the future this will first be passed to dbus, for user approval
        let initiate = InitiateParams::from_user_message(&message, &services);

        match rvi::send_message(&rvi_url, initiate, &message.services.start) {
            Ok(..) => {},
            Err(e) => { error!("Couldn't initiate download: {}", e); }
        }
    }
}
