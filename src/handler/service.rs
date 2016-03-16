//! The main service handler
//!
//! Parses incoming messages and delegates them to the appropriate individual message handlers,
//! passing on the results to the [`main_loop`](../main_loop/index.html)

use jsonrpc;
use jsonrpc::{OkResponse, ErrResponse};

use std::io::{Read, Write};
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
use std::thread;
use std::thread::sleep_ms;

use time;
use hyper::server::{Handler, Request, Response};
use rustc_serialize::{json, Decodable};
use rustc_serialize::json::Json;

use rvi;
use rvi::{Message, ServiceEdge};

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

    pub fn start(self, edge: ServiceEdge) -> LocalServices {
        edge.register_service("/sota/notify");
        let svcs = LocalServices {
            start: edge.register_service("/sota/start"),
            chunk: edge.register_service("/sota/chunk"),
            abort: edge.register_service("/sota/abort"),
            finish: edge.register_service("/sota/finish"),
            getpackages: edge.register_service("/sota/getpackages")
        };
        self.remote_services.lock().unwrap().vin = svcs.get_vin(self.conf.client.vin_match);
        thread::spawn(move || edge.start(self));
        svcs
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
        json::decode::<jsonrpc::Request<Message<D>>>(&message)
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

    /// Try to parse the type of a message and forward it to the appropriate message handler.
    /// Returns the result of the message handling or a `jsonrpc` result indicating a parser error.
    ///
    /// Needs to be extended to support new services.
    ///
    /// # Arguments
    /// * `message`: The message that will be parsed.
    fn handle_message(&self, message: &str)
        -> Result<OkResponse<i32>, ErrResponse> {
        macro_rules! handle_params {
            ($handler:ident, $message:ident, $service:ident, $id:ident,
             $( $x:ty, $i:expr), *) => {{
                $(
                    if $i == $service {
                        return $handler.handle_message_params::<$x>($id, $message)
                    }
                )*
            }}
        }

        let data = try!(Json::from_str(message)
                        .map_err(|_| ErrResponse::parse_error()));
        let obj = try!(data.as_object().ok_or(ErrResponse::parse_error()));
        let rpc_id = try!(obj.get("id").and_then(|x| x.as_u64())
                          .ok_or(ErrResponse::parse_error()));

        let method = try!(obj.get("method").and_then(|x| x.as_string())
                          .ok_or(ErrResponse::invalid_request(rpc_id)));

        if method == "services_available" {
            Ok(OkResponse::new(rpc_id, None))
        }
        else if method != "message" {
            Err(ErrResponse::method_not_found(rpc_id))
        } else {
            let service = try!(obj.get("params")
                               .and_then(|x| x.as_object())
                               .and_then(|x| x.get("service_name"))
                               .and_then(|x| x.as_string())
                               .ok_or(ErrResponse::invalid_request(rpc_id)));

            handle_params!(self, message, service, rpc_id,
                           NotifyParams, "/sota/notify",
                           StartParams,  "/sota/start",
                           ChunkParams,  "/sota/chunk",
                           FinishParams, "/sota/finish",
                           ReportParams, "/sota/getpackages",
                           AbortParams,  "/sota/abort");

            Err(ErrResponse::invalid_request(rpc_id))
        }
    }
}

impl Handler for ServiceHandler {
    fn handle(&self, mut req: Request, resp: Response) {
        let mut rbody = String::new();
        try_or!(req.read_to_string(&mut rbody), return);
        debug!(">>> Received Message: {}", rbody);
        let mut resp = try_or!(resp.start(), return);

        macro_rules! send_response {
            ($rtype:ty, $resp:ident) => {
                match json::encode::<$rtype>(&$resp) {
                    Ok(decoded_msg) => {
                        try_or!(resp.write_all(decoded_msg.as_bytes()), return);
                        debug!("<<< Sent Response: {}", decoded_msg);
                    },
                    Err(p) => { error!("{}", p); }
                }
            };
        }

        match self.handle_message(&rbody) {
            Ok(msg) => { send_response!(OkResponse<i32>, msg) },
            Err(msg) => { send_response!(ErrResponse, msg) }
        }

        try_or!(resp.end(), return);
    }
}
