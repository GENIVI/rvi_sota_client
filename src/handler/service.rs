use jsonrpc;
use jsonrpc::{OkResponse, ErrResponse};

use std::io::{Read, Write};
use std::sync::Mutex;
use std::sync::mpsc::Sender;

use hyper::server::{Handler, Request, Response};
use rustc_serialize::{json, Decodable};
use rustc_serialize::json::Json;

use std::collections::HashMap;

use rvi::{Message, RVIHandler, Service};

use message::{BackendServices, PackageId, UserMessage, LocalServices};
use handler::{NotifyParams, StartParams, ChunkParams, FinishParams};
use handler::HandleMessageParams;
use persistence::Transfer;

pub struct ServiceHandler {
    rvi_url: String,
    sender: Mutex<Sender<UserMessage>>,
    services: Mutex<BackendServices>,
    transfers: Mutex<HashMap<PackageId, Transfer>>,
    storage_dir: String,
    vin: String
}

impl ServiceHandler {
    pub fn new(sender: Sender<UserMessage>,
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
            transfers: Mutex::new(HashMap::new()),
            vin: String::new(),
            storage_dir: dir
        }
    }

    fn push_notify(&self, message: UserMessage) {
        try_or!(self.sender.lock().unwrap().send(message), return);
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
            handler.get_message().map(|m| { self.push_notify(m); });

            if result {
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
        // TODO: refactor
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

        macro_rules! try_or_parse_error {
            ($run:expr) => {
                match $run {
                    Some(val) => val,
                    None =>  return Err(ErrResponse::parse_error())
                }
            }
        }

        macro_rules! try_or_invalid {
            ($run:expr, $id:ident) => {
                match $run {
                    Some(val) => val,
                    None => return Err(ErrResponse::invalid_request($id))
                }
            }
        }

        let data = try!(Json::from_str(message).map_err(|_| ErrResponse::parse_error()));
        let obj = try_or_parse_error!(data.as_object());
        let rpc_id_data = try_or_parse_error!(obj.get("id"));
        let rpc_id = try_or_parse_error!(rpc_id_data.as_u64());

        let method_data = try_or_invalid!(obj.get("method"), rpc_id);
        let method = try_or_invalid!(method_data.as_string(), rpc_id);

        if method == "services_available" {
            Ok(OkResponse::new(rpc_id, None))
        }
        else if method != "message" {
            Err(ErrResponse::method_not_found(rpc_id))
        } else {
            let params_data = try_or_invalid!(obj.get("params"), rpc_id);
            let params = try_or_invalid!(params_data.as_object(), rpc_id);
            let service_data = try_or_invalid!(params.get("service_name"), rpc_id);
            let service = try_or_invalid!(service_data.as_string(), rpc_id);

            handle_params!(self, message, service, rpc_id,
                           NotifyParams, "/sota/notify",
                           StartParams,  "/sota/start",
                           ChunkParams,  "/sota/chunk",
                           FinishParams, "/sota/finish");

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
