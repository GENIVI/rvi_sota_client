use jsonrpc;
use jsonrpc::{OkResponse, ErrResponse};

use std::io::{Read, Write};
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
use std::thread::sleep_ms;

use time;
use hyper::server::{Handler, Request, Response};
use rustc_serialize::{json, Decodable};
use rustc_serialize::json::Json;

use std::collections::HashMap;

use rvi::{Message, RVIHandler, Service};

use message::{BackendServices, PackageId, LocalServices, Notification};
use handler::{NotifyParams, StartParams, ChunkParams, FinishParams};
use handler::{ReportParams, AbortParams, HandleMessageParams};
use persistence::Transfer;

pub struct ServiceHandler {
    rvi_url: String,
    sender: Mutex<Sender<Notification>>,
    services: Mutex<BackendServices>,
    transfers: Arc<Mutex<HashMap<PackageId, Transfer>>>,
    storage_dir: String,
    vin: String
}

impl ServiceHandler {
    pub fn new(transfers: Arc<Mutex<HashMap<PackageId, Transfer>>>,
               sender: Sender<Notification>,
               url: String, dir: String) -> ServiceHandler {
        let services = BackendServices {
            start: String::new(),
            cancel: String::new(),
            ack: String::new(),
            report: String::new()
        };

        ServiceHandler {
            rvi_url: url,
            sender: Mutex::new(sender),
            services: Mutex::new(services),
            transfers: transfers,
            vin: String::new(),
            storage_dir: dir
        }
    }

    pub fn start_timer(transfers: &Mutex<HashMap<PackageId, Transfer>>,
                       timeout: i64) {
        loop {
            sleep_ms(1000);
            let time_now = time::get_time().sec;
            let mut transfers = transfers.lock().unwrap();

            let mut timed_out = Vec::new();
            for transfer in transfers.deref_mut() {
                if time_now - transfer.1.last_chunk_received > timeout {
                    timed_out.push(transfer.0.clone());
                }
            }

            for transfer in timed_out {
                info!("Transfer for package {} timed out after {} ms",
                      transfer, timeout);
                let _ = transfers.remove(&transfer);
            }
        }
    }

    fn push_notify(&self, m: Notification) {
        try_or!(self.sender.lock().unwrap().send(m), return);
    }

    fn handle_message_params<D>(&self, message: &str)
        -> Option<Result<OkResponse<i32>, ErrResponse>>
        where D: Decodable + HandleMessageParams {
        json::decode::<jsonrpc::Request<Message<D>>>(&message).map(|p| {
            let handler = &p.params.parameters[0];
            let result = handler.handle(&self.services,
                                        &self.transfers,
                                        &self.rvi_url,
                                        &self.vin,
                                        &self.storage_dir);
            if result {
                handler.get_message().map(|m| { self.push_notify(m); });
                Ok(OkResponse::new(p.id, None))
            } else {
                // TODO: don't just return true/false, but the actual error,
                // so we can send apropriate responses back.
                Err(ErrResponse::invalid_request(p.id))
            }
        }).ok()
    }

    fn handle_message(&self, message: &str)
        -> Result<OkResponse<i32>, ErrResponse> {
        macro_rules! handle_params {
            ($handler:ident, $message:ident, $service:ident, $id:ident,
             $( $x:ty, $i:expr), *) => {{
                $(
                    if $i == $service {
                        match $handler.handle_message_params::<$x>($message) {
                            Some(r) => return r,
                            None => return Err(ErrResponse::invalid_params($id))
                        }
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
                           ReportParams, "/sota/report",
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

impl RVIHandler for ServiceHandler {
    fn register(&mut self, services: Vec<Service>) {
        self.vin = LocalServices::new(&services).get_vin();
    }
}