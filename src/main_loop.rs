use std::sync::mpsc::channel;
use std::thread;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::ops::Deref;

use rvi;
use handler::ServiceHandler;
use message::{InitiateParams, BackendServices, PackageId};
use message::{Notification, ServerPackageReport, LocalServices, ServerReport};
use configuration::Configuration;
use persistence::Transfer;
use sota_dbus;

/// Start a SOTA client service with the provided configuration
pub fn start(conf: &Configuration, rvi_url: String, edge_url: String) {
    // will receive RVI registration details
    let (tx_edge, rx_edge) = channel();
    let rvi_edge = rvi::ServiceEdge::new(rvi_url.clone(),
                                         edge_url.clone(),
                                         tx_edge);

    let transfers: Arc<Mutex<HashMap<PackageId, Transfer>>> =
        Arc::new(Mutex::new(HashMap::new()));

    // will receive notifies from RVI and install requests from dbus
    let (tx_main, rx_main) = channel();
    let handler = ServiceHandler::new(transfers.clone(), tx_main.clone(),
                                      rvi_url.clone(), conf.clone());

    match conf.client.timeout {
        Some(timeout) => {
            let _ = thread::spawn(move || {
                ServiceHandler::start_timer(transfers.deref(), timeout);
            });
        },
        None => info!("No timeout configured, transfers will never time out.")
    }

    let services = vec!("/sota/notify",
                        "/sota/start",
                        "/sota/chunk",
                        "/sota/finish",
                        "/sota/getpackages",
                        "/sota/abort");

    thread::spawn(move || {
        rvi_edge.start(handler, services);
    });

    let dbus_receiver = sota_dbus::Receiver::new(conf.dbus.clone(),
                                                 tx_main.clone());
    thread::spawn(move || {
        dbus_receiver.start();
    });

    let local_services = LocalServices::new(&rx_edge.recv().unwrap());
    let mut backend_services = BackendServices::new();

    loop {
        match rx_main.recv().unwrap() {
            Notification::Notify(notify) => {
                backend_services.update(&notify.services);
                sota_dbus::send_notify(&conf.dbus, notify.packages);
            },
            Notification::Initiate(packages) => {
                let initiate =
                    InitiateParams::new(packages, local_services.clone(),
                                        local_services
                                        .get_vin(conf.client.vin_match));
                match rvi::send_message(&rvi_url, initiate,
                                        &backend_services.start) {
                    Ok(..) => {},
                    Err(e) => error!("Couldn't initiate download: {}", e)
                }
            },
            Notification::Finish(package) => {
                let report = sota_dbus::request_install(&conf.dbus, package);
                let server_report =
                    ServerPackageReport::new(report, local_services
                                             .get_vin(conf.client.vin_match));

                match rvi::send_message(&rvi_url, server_report,
                                        &backend_services.packages) {
                    Ok(..) => {},
                    Err(e) => error!("Couldn't send report: {}", e)
                }
            },
            Notification::Report => {
                let packages = sota_dbus::request_report(&conf.dbus);
                let report =
                    ServerReport::new(packages, local_services
                                      .get_vin(conf.client.vin_match));

                match rvi::send_message(&rvi_url, report,
                                        &backend_services.packages) {
                    Ok(..) => {},
                    Err(e) => error!("Couldn't send report: {}", e)
                }
            }
        }
    }
}
