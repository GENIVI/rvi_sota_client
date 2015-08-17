// TODO: Remove MessageEvents, is now soley handled by MessageEventParams
// TODO: DRY up HandleMessageEvent implementations
// TODO: Send proper messages to the channel on the main event loop
// TODO: Add error handling, remove `unwrap()`
// TODO: WRITE FUCKING TESTS!!!!
// TODO: drop json_rpc responses

mod message;

use jsonrpc;
use jsonrpc::{OkResponse, ErrResponse};

use std::io::{Read, Write};
use std::sync::Mutex;
use std::sync::mpsc::Sender;

use std::vec::Vec;

use hyper::{Client, Server};
use hyper::server::{Handler, Request, Response};
use url::Url;
use rustc_serialize::{json, Decodable, Encodable};

use std::collections::HashMap;
use persistence::PackageFile;

use rvi::message::*;

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

    // TODO: refactor into some sending module or something
    fn send_ack(&self, ack: GenericAck) {
        let client = Client::new();
        let mut json_body: String;

        match ack {
            GenericAck::Ack(p) => {
                let mut message = Message::<AckParams> {
                    service_name: "genivi.org/backend/sota/ack".to_string(),
                    parameters: Vec::new()
                };
                message.parameters.push(p);
                let json_rpc = jsonrpc::Request::new("message", message);
                json_body = json::encode(&json_rpc).unwrap();
            },
            GenericAck::Chunk(p) => {
                let mut message = Message::<AckChunkParams> {
                    service_name: "genivi.org/backend/sota/ack".to_string(),
                    parameters: Vec::new()
                };
                message.parameters.push(p);
                let json_rpc = jsonrpc::Request::new("message", message);
                json_body = json::encode(&json_rpc).unwrap();
            }
        }

        println!("<<< Sent Message: {}", json_body);
        let mut resp = client.post(self.rvi_url.clone())
            .body(&json_body)
            .send()
            .unwrap();

        let mut rbody = String::new();
        resp.read_to_string(&mut rbody).unwrap();
        println!(">>> Received Message: {}", rbody);
    }
}

impl Handler for RviServiceHandler {
    fn handle(&self, mut req: Request, resp: Response) {
        let mut rbody = String::new();
        req.read_to_string(&mut rbody).unwrap();
        println!(">>> Received Message: {}", rbody);
        let mut resp = resp.start().unwrap();

        match self.handle_message(&rbody) {
            Ok(response) => {
                match json::encode::<OkResponse>(&response) {
                    Ok(decoded_msg) => {
                        resp.write_all(decoded_msg.as_bytes()).unwrap();
                        println!("<<< Sent Message: {}", decoded_msg);
                    },
                    Err(p) => { println!("ERROR: {}", p); }
                }
            },
            Err(msg) => {
                match json::encode::<ErrResponse>(&msg) {
                    Ok(decoded_msg) => {
                        resp.write_all(decoded_msg.as_bytes()).unwrap();
                        println!("<<< Sent Message: {}", decoded_msg);
                    },
                    Err(p) => { println!("ERROR: {}", p); }
                }
            }
        }

        resp.end().unwrap();
    }
}

pub struct RviServiceEdge {
    client: Client,
    rvi_url: Url,
    edge_url: Url
}

impl RviServiceEdge {
    pub fn new(r: Url, e: Url) -> RviServiceEdge {
        RviServiceEdge {
            client: Client::new(),
            rvi_url: r,
            edge_url: e
        }
    }

    pub fn send<E: Encodable>(&self, b: &E) {
        let json_body = json::encode(b).unwrap();
        println!("<<< Sent Message: {}", json_body);
        let mut resp = self.client.post(self.rvi_url.clone())
            .body(&json_body)
            .send()
            .unwrap();

        let mut rbody = String::new();
        resp.read_to_string(&mut rbody).unwrap();
        println!(">>> Received Message: {}", rbody);
    }

    pub fn register_service(&self, s: &str) {
        let json_rpc = jsonrpc::Request::new(
            "register_service",
            RegisterServiceParams {
                network_address: self.edge_url.to_string(),
                service: s.to_string()
            });
        self.send(&json_rpc);
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
        srv.handle(h).unwrap();
    }
}

// TODO: refactor into some sending module or something
pub fn initiate_download(rvi_url: Url, package: String) {
    let client = Client::new();
    let params = InitiateParams{
        id: 1, // TODO: implement a incremental id thingy (probably based on pending_packages)
        package: package
    };

    let mut message = Message::<InitiateParams> {
        service_name: "genivi.org/backend/sota/initiate_download".to_string(),
        parameters: Vec::new()
    };
    message.parameters.push(params);
    let json_rpc = jsonrpc::Request::new("message", message);
    let json_body = json::encode(&json_rpc).unwrap();

    println!("<<< Sent Message: {}", json_body);
    let mut resp = client.post(rvi_url.clone())
        .body(&json_body)
        .send()
        .unwrap();

    let mut rbody = String::new();
    resp.read_to_string(&mut rbody).unwrap();
    println!(">>> Received Message: {}", rbody);
}
