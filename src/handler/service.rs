//! The main service handler
//!
//! Parses incoming messages and delegates them to the appropriate individual message handlers,
//! passing on the results to the [`main_loop`](../main_loop/index.html)

use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
use std::thread;
use std::thread::sleep_ms;

use rustc_serialize::{json, Decodable};
use time;

use jsonrpc;
use jsonrpc::{OkResponse, ErrResponse};
use rvi;

use super::ChunkReceived;
use event::Event;
use event::inbound::InboundEvent;
// use message::ServerPackageReport;
use handler::{NotifyParams, StartParams, ChunkParams, FinishParams};
use handler::{ReportParams, AbortParams, HandleMessageParams};
use persistence::Transfers;
use configuration::Configuration;

/// Encodes the list of service URLs the client registered.
///
/// Needs to be extended to introduce new services.
#[derive(RustcEncodable, Clone)]
pub struct LocalServices {
    /// "Start Download" URL.
    pub start: String,
    /// "Chunk" URL.
    pub chunk: String,
    /// "Abort Download" URL.
    pub abort: String,
    /// "Finish Download" URL.
    pub finish: String,
    /// "Get All Packages" URL.
    pub getpackages: String,
}

impl LocalServices {
    /// Returns the VIN of this device.
    ///
    /// # Arguments
    /// * `vin_match`: The index, where to look for the VIN in the service URL.
    pub fn get_vin(&self, vin_match: i32) -> String {
        self.start.split("/").nth(vin_match as usize).unwrap().to_string()
    }
}

/// Encodes the service URLs, that the server provides.
#[derive(RustcDecodable, Clone)]
pub struct BackendServices {
    /// URL for the "Start Download" call.
    pub start: String,
    /// URL for the "Chunk Received" call.
    pub ack: String,
    /// URL for the "Installation Report" call.
    pub report: String,
    /// URL for the "Get All Packages" call.
    pub packages: String
}

impl BackendServices {
    /// Creates a new, empty `BackendServices` object.
    pub fn new() -> BackendServices {
        BackendServices {
            start: "".to_string(),
            ack: "".to_string(),
            report: "".to_string(),
            packages: "".to_string()
        }
    }
}

pub struct RemoteServices {
    pub vin: String,
    url: String,
    pub svcs: Option<BackendServices>
}

impl RemoteServices {
    pub fn new(url: String) -> RemoteServices {
        RemoteServices {
            vin: String::new(),
            url: url,
            svcs: None
        }
    }

    pub fn set(&mut self, svcs: BackendServices) {
        self.svcs = Some(svcs);
    }

    pub fn send_chunk_received(&self, m: ChunkReceived) -> Result<String, String> {
        self.svcs.iter().next().ok_or(format!("RemoteServices not set"))
            .and_then(|ref svcs| rvi::send_message(&self.url, m, &svcs.ack))
    }

    /*
    pub fn send_package_report(&self, m: ServerPackageReport) -> Result<String, String> {
        self.svcs.iter().next().ok_or(format!("RemoteServices not set"))
            .and_then(|ref svcs| rvi::send_message(&self.url, m, &svcs.report))
    }
    */
}


/// Type that encodes a single service handler.
///
/// Holds the necessary state, like in-progress transfers, that are needed for handling incoming
/// messages and sending replies to RVI. Needs to be thread safe as
/// [`hyper`](../../../hyper/index.html) handles requests asynchronously.
pub struct ServiceHandler {
    /// A `Sender` that connects the handlers with the `main_loop`.
    sender: Mutex<Sender<Event>>,
    /// The currently in-progress `Transfer`s.
    transfers: Arc<Mutex<Transfers>>,
    /// The service URLs that the SOTA server advertised.
    remote_services: Mutex<RemoteServices>,
    /// The full `Configuration` of sota_client.
    conf: Configuration
}

impl ServiceHandler {
    /// Create a new `ServiceHandler`.
    ///
    /// # Arguments
    /// * `transfers`: A `Transfers` object to store the in-progress `Transfer`s.
    /// * `sender`: A `Sender` to call back into the `main_loop`.
    /// * `url`: The full URL, where RVI can be reached.
    /// * `c`: The full `Configuration` of sota_client.
    pub fn new(sender: Sender<Event>,
               url: String,
               c: Configuration) -> ServiceHandler {
        let transfers = Arc::new(Mutex::new(Transfers::new(c.client.storage_dir.clone())));
        let tc = transfers.clone();
        c.client.timeout
            .map(|t| {
                let _ = thread::spawn(move || ServiceHandler::start_timer(tc.deref(), t));
                info!("Transfers timeout after {}", t)})
            .unwrap_or(info!("No timeout configured, transfers will never time out."));

        ServiceHandler {
            sender: Mutex::new(sender),
            transfers: transfers,
            remote_services: Mutex::new(RemoteServices::new(url)),
            conf: c
        }
    }

    /// Starts a infinite loop to expire timed out transfers. Checks once a second for timed out
    /// transfers.
    ///
    /// # Arguments
    /// * `transfers`: Pointer to a `Transfers` object, that stores the transfers to be checked for
    ///   expired timeouts.
    /// * `timeout`: The timeout in seconds.
    pub fn start_timer(transfers: &Mutex<Transfers>,
                       timeout: i64) {
        loop {
            sleep_ms(1000);
            let mut transfers = transfers.lock().unwrap();
            transfers.prune(time::get_time().sec, timeout);
        }
    }

    /// Helper function to send a `Event` to the `main_loop`.
    ///
    /// # Arguments
    /// * `e`: `Event` to send.
    fn push_notify(&self, e: InboundEvent) {
        try_or!(self.sender.lock().unwrap().send(Event::Inbound(e)), return);
    }

    /// Create a message handler `D`, and let it process the `message`. If it returns a
    /// Event, forward it to the `main_loop`. Returns a `jsonrpc` response indicating
    /// success or failure.
    ///
    /// # Arguments
    /// * `message`: The message, that should be handled.
    fn handle_message_params<D>(&self, id: u64, message: &str)
        -> Result<OkResponse<i32>, ErrResponse>
        where D: Decodable + HandleMessageParams {
        json::decode::<jsonrpc::Request<rvi::Message<D>>>(&message)
            .map_err(|_| ErrResponse::invalid_params(id))
            .and_then(|p| {
                let handler = &p.params.parameters[0];
                handler.handle(&self.remote_services, &self.transfers)
                    .map_err(|_| ErrResponse::unspecified(p.id))
                    .map(|r| {
                        r.map(|m| self.push_notify(m));
                        OkResponse::new(p.id, None) })
            })
        }
}

impl rvi::ServiceHandler for ServiceHandler {
    fn handle_service(&self, id: u64, service: &str, message: &str)
        -> Result<OkResponse<i32>, ErrResponse> {
        match service {
            "/sota/notify" => self.handle_message_params::<NotifyParams>(id, message),
            "/sota/start" => self.handle_message_params::<StartParams>(id, message),
            "/sota/chunk" => self.handle_message_params::<ChunkParams>(id, message),
            "/sota/finish" => self.handle_message_params::<FinishParams>(id, message),
            "/sota/abort" => self.handle_message_params::<AbortParams>(id, message),
            "/sota/getpackages" => self.handle_message_params::<ReportParams>(id, message),
            _ => Err(ErrResponse::invalid_request(id))
        }
    }

    fn register_services<F: Fn(&str) -> String>(&self, reg: F) {
        reg("/sota/notify");
        let svcs = LocalServices {
            start: reg("/sota/start"),
            chunk: reg("/sota/chunk"),
            abort: reg("/sota/abort"),
            finish: reg("/sota/finish"),
            getpackages: reg("/sota/getpackages")
        };
        self.remote_services.lock().unwrap().vin = svcs.get_vin(self.conf.client.vin_match);
    }
}

