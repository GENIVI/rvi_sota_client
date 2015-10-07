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
    let handler = ServiceHandler::new(transfers.clone(),
                                      tx_main.clone(), rvi_url.clone(),
                                      conf.client.storage_dir.clone());

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

    let (tx_dbus, rx_dbus) = channel();
    let dbus_sender = sota_dbus::Sender::new(conf.dbus.clone(),
                                             rx_dbus, tx_main.clone());
    thread::spawn(move || {
        dbus_sender.start();
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
                let message = sota_dbus::Request::Notify(notify.packages);
                let _ = tx_dbus.send(message);
            },
            Notification::Initiate(packages) => {
                let initiate =
                    InitiateParams::new(packages, local_services.clone());
                match rvi::send_message(&rvi_url, initiate,
                                        &backend_services.start) {
                    Ok(..) => {},
                    Err(e) => error!("Couldn't initiate download: {}", e)
                }
            }
            Notification::Finish(package) => {
                tx_dbus.send(sota_dbus::Request::Complete(package)).unwrap();
            },
            Notification::RequestReport => {
                tx_dbus.send(sota_dbus::Request::Report).unwrap();
            },
            Notification::InstallReport(report) => {
                let report =
                    ServerPackageReport::new(report, local_services.get_vin());
                match rvi::send_message(&rvi_url, report,
                                        &backend_services.report) {
                    Ok(..) => {},
                    Err(e) => error!("Couldn't send install report: {}", e)
                }
            },
            Notification::Report(report) => {
                let report =
                    ServerReport::new(report, local_services.get_vin());
                match rvi::send_message(&rvi_url, report,
                                        &backend_services.packages) {
                    Ok(..) => {},
                    Err(e) => error!("Couldn't send report: {}", e)
                }
            }
        }
    }
}
