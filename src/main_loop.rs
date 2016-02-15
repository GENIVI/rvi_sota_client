//! Main loop, starting the worker threads and wiring up communication channels between them.

use std::sync::mpsc::channel;
use std::thread;

use rvi;
use handler::ServiceHandler;
use message::{InitiateParams, BackendServices, Notification, ServerPackageReport, ServerReport};
use configuration::Configuration;
use sota_dbus;

/// Main loop, starting the worker threads and wiring up communication channels between them.
///
/// # Arguments
/// * `conf`: A pointer to a `Configuration` object see the [documentation of the configuration
///   crate](../configuration/index.html).
/// * `rvi_url`: The URL, where RVI can be found, with the protocol.
/// * `edge_url`: The `host:port` combination where the client should bind and listen for incoming
///   RVI calls.
pub fn start(conf: &Configuration, rvi_url: String, edge_url: String) {
    // Main message channel from RVI and DBUS
    let (tx_main, rx_main) = channel();

    // RVI edge handler
    let handler = ServiceHandler::new(tx_main.clone(), rvi_url.clone(), conf.clone());
    let rvi_edge = rvi::ServiceEdge::new(rvi_url.clone(), edge_url);
    let local_services = handler.start(rvi_edge);

    // DBUS handler
    let dbus_receiver = sota_dbus::Receiver::new(conf.dbus.clone(), tx_main);
    thread::spawn(move || dbus_receiver.start());

    let mut backend_services = BackendServices::new();

    loop {
        match rx_main.recv().unwrap() {
            // Pass on notifications to the DBus
            Notification::Notify(notify) => {
                backend_services.update(&notify.services);
                sota_dbus::send_notify(&conf.dbus, notify.packages);
            },
            // Pass on initiate requests to RVI
            Notification::Initiate(packages) => {
                let initiate = InitiateParams::new(
                    packages,
                    local_services.clone(),
                    local_services.get_vin(conf.client.vin_match));
                rvi::send_message(&rvi_url, initiate, &backend_services.start)
                    .map_err(|e| error!("Couldn't initiate download: {}", e))
                    .unwrap();
            },
            // Request and forward the installation report from DBus to RVI.
            Notification::Finish(package) => {
                let report = sota_dbus::request_install(&conf.dbus, package);
                let server_report = ServerPackageReport::new(
                    report, local_services.get_vin(conf.client.vin_match));
                rvi::send_message(&rvi_url, server_report, &backend_services.report)
                    .map_err(|e| error!("Couldn't send report: {}", e))
                    .unwrap();
            },
            // Request a full report via DBus and forward it to RVI
            Notification::Report => {
                let packages = sota_dbus::request_report(&conf.dbus);
                let report = ServerReport::new(
                    packages, local_services.get_vin(conf.client.vin_match));
                rvi::send_message(&rvi_url, report, &backend_services.packages)
                    .map_err(|e| error!("Couldn't send report: {}", e))
                    .unwrap();
            }
        }
    }
}
