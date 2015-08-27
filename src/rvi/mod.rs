// TODO: Send proper messages to the channel on the main event loop
// TODO: Add error handling, remove `unwrap()`
// TODO: WRITE FUCKING TESTS!!!!
// TODO: drop json_rpc responses
// TODO: Only send full messages when debugging is on

mod message;
mod send_msg;

use jsonrpc;
use jsonrpc::{OkResponse, ErrResponse};

use std::io::{Read, Write};
use std::sync::Mutex;
use std::sync::mpsc::Sender;

use std::vec::Vec;

use hyper::Server;
use hyper::server::{Handler, Request, Response};
use url::Url;
use rustc_serialize::{json, Decodable, Encodable};

use std::collections::HashMap;
use persistence::PackageFile;

use rvi::message::*;
use rvi::send_msg::send;

#[derive(RustcEncodable)]
struct RegisterServiceParams {
    network_address: String,
    service: String
}

pub struct RviServiceHandler {
    sender: Mutex<Sender<String>>,
    pending: Mutex<HashMap<String, i32>>,
    transfers: Mutex<HashMap<u32, PackageFile>>,
    rvi_url: Url,
}

impl RviServiceHandler {
    pub fn new(s: Sender<String>, u: Url) -> RviServiceHandler {
        RviServiceHandler {
            sender: Mutex::new(s),
            rvi_url: u,
            pending: Mutex::new(HashMap::new()),
            transfers: Mutex::new(HashMap::new()),
        }
    }

    fn push_notify(&self, e: String) {
        self.sender.lock().unwrap().send(e).unwrap();
    }

    fn handle_message_params<D>(&self, b: &String)
        -> Option<Result<OkResponse, ErrResponse>>
        where D: Decodable + HandleMessageParams {
        match json::decode::<jsonrpc::Request<Message<D>>>(&b) {
            Ok(p) => {
                let handler = &p.params.parameters[0];
                let result = handler.handle(&self.pending, &self.transfers);

                if p.params.service_name == "/sota/notify" {
                    self.push_notify(p.params.parameters[0].get_message());
                }

                match handler.get_ack() {
                    Some(a) => { self.send_ack(a); }
                    None => {}
                }

                if result {
                    Some(Ok(OkResponse::new(p.id, None)))
                } else {
                    Some(Err(ErrResponse::invalid_request(p.id)))
                }
            },
            _ => {None}
        }
    }

    fn handle_message(&self, b: &String)
        -> Result<OkResponse, ErrResponse> {
        // TODO: Parse JSON-RPC, without multiple matches on parameters
        match self.handle_message_params::<NotifyParams>(b) {
            Some(result) => {return result;},
            None => {}
        }
        match self.handle_message_params::<StartParams>(b) {
            Some(result) => {return result;},
            None => {}
        }
        match self.handle_message_params::<ChunkParams>(b) {
            Some(result) => {return result;},
            None => {}
        }
        match self.handle_message_params::<FinishParams>(b) {
            Some(result) => {return result;},
            None => {}
        }
        match json::decode::<jsonrpc::Request<Message<String>>>(b) {
            Ok(p) => {
                Err(ErrResponse::method_not_found(p.id))
            },
            _ => {
                Err(ErrResponse::parse_error())
            }
        }
    }

    // TODO: DRY up
    fn send_ack(&self, ack: GenericAck) {
        match ack {
            GenericAck::Ack(p) => {
                let mut message = Message::<AckParams> {
                    service_name: "genivi.org/backend/sota/ack".to_string(),
                    parameters: Vec::new()
                };
                message.parameters.push(p);
                let json_rpc = jsonrpc::Request::new("message", message);
                send(self.rvi_url.clone(), &json_rpc);
            },
            GenericAck::Chunk(p) => {
                let mut message = Message::<AckChunkParams> {
                    service_name: "genivi.org/backend/sota/ack".to_string(),
                    parameters: Vec::new()
                };
                message.parameters.push(p);
                let json_rpc = jsonrpc::Request::new("message", message);
                send(self.rvi_url.clone(), &json_rpc);
            }
        }
    }
}

impl Handler for RviServiceHandler {
    fn handle(&self, mut req: Request, resp: Response) {
        let mut rbody = String::new();
        req.read_to_string(&mut rbody).unwrap();
        debug!(">>> Received Message: {}", rbody);
        let mut resp = resp.start().unwrap();

        match self.handle_message(&rbody) {
            Ok(response) => {
                match json::encode::<OkResponse>(&response) {
                    Ok(decoded_msg) => {
                        resp.write_all(decoded_msg.as_bytes()).unwrap();
                        debug!("<<< Sent Response: {}", decoded_msg);
                    },
                    Err(p) => { error!("!!! ERR: {}", p); }
                }
            },
            Err(msg) => {
                match json::encode::<ErrResponse>(&msg) {
                    Ok(decoded_msg) => {
                        resp.write_all(decoded_msg.as_bytes()).unwrap();
                        debug!("<<< Sent Response: {}", decoded_msg);
                    },
                    Err(p) => { error!("!!! ERR: {}", p); }
                }
            }
        }

        resp.end().unwrap();
    }
}

pub struct RviServiceEdge {
    rvi_url: Url,
    edge_url: Url
}

impl RviServiceEdge {
    pub fn new(r: Url, e: Url) -> RviServiceEdge {
        RviServiceEdge {
            rvi_url: r,
            edge_url: e
        }
    }

    pub fn register_service(&self, s: &str) {
        let json_rpc = jsonrpc::Request::new(
            "register_service",
            RegisterServiceParams {
                network_address: self.edge_url.to_string(),
                service: s.to_string()
            });
        send(self.rvi_url.clone(), &json_rpc);
    }

    pub fn start(&self, h: RviServiceHandler) {
        self.register_service("/sota/notify");
        self.register_service("/sota/start");
        self.register_service("/sota/chunk");
        self.register_service("/sota/finish");

        let addr = (
            &*self.edge_url.host().unwrap().to_string(),
            self.edge_url.port().unwrap());
        let srv = Server::http(addr).unwrap();

        info!("Ready to accept connections.");
        srv.handle(h).unwrap();
    }
}

pub fn initiate_download(rvi_url: Url, package: String) {
    let mut message = Message::<InitiateParams> {
        service_name: "genivi.org/backend/sota/initiate_download".to_string(),
        parameters: Vec::new()
    };

    let params = InitiateParams{
        id: 1, // TODO: Get the correct ID from Service Edge
        package: package
    };

    message.parameters.push(params);
    let json_rpc = jsonrpc::Request::new("message", message);
    send(rvi_url.clone(), &json_rpc);
}
