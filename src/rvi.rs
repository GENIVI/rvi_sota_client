use jsonrpc;

use std::io::{Read, Write};
use std::sync::Mutex;
use std::sync::mpsc::Sender;
use std::vec;

use hyper::{Client, Server};
use hyper::server::{Handler, Request, Response};
use url::Url;
use rustc_serialize::{json, Decodable, Encodable};


/// TODO: Add error handling, remove `unwrap()`

#[derive(RustcEncodable)]
struct RegisterServiceParams {
    network_address: String,
    service: String
}

#[derive(RustcDecodable)]
struct MessageParams<T> {
    service_name: String,
    parameters: vec::Vec<T>
}

#[derive(RustcDecodable, Clone)]
pub struct NotifyParams {
    pub retry: i32,
    pub package: String
}

#[derive(RustcDecodable, Clone)]
pub struct StartParams {
    pub total_size: i32,
    pub package: String,
    pub chunk_size: i32
}

#[derive(RustcDecodable, Clone)]
pub struct ChunkParams {
    pub index: i32,
    pub msg: String
}

#[derive(RustcDecodable, Clone)]
pub struct FinishParams {
    dummy: i32
}

pub enum MessageEventParams {
    Notify(NotifyParams),
    Start(StartParams),
    Chunk(ChunkParams),
    Finish(FinishParams)
}

pub struct MessageEvent {
    pub service_name: String,
    pub message_id: u64,
    pub params: MessageEventParams
}

trait ToMessageEvent {
    fn to_event(self, s: String, id: u64) -> MessageEvent;
}

impl ToMessageEvent for NotifyParams {
    fn to_event(self, s: String, id: u64) -> MessageEvent {
        MessageEvent {
            service_name: s,
            message_id: id,
            params: MessageEventParams::Notify(self)}
    }
}

impl ToMessageEvent for StartParams {
    fn to_event(self, s: String, id: u64) -> MessageEvent {
        MessageEvent {
            service_name: s,
            message_id: id,
            params: MessageEventParams::Start(self)}
    }
}

impl ToMessageEvent for ChunkParams {
    fn to_event(self, s: String, id: u64) -> MessageEvent {
        MessageEvent {
            service_name: s,
            message_id: id,
            params: MessageEventParams::Chunk(self)}
    }
}

impl ToMessageEvent for FinishParams {
    fn to_event(self, s: String, id: u64) -> MessageEvent {
        MessageEvent {
            service_name: s,
            message_id: id,
            params: MessageEventParams::Finish(self)}
    }
}


pub struct RviServiceHandler {
    sender: Mutex<Sender<MessageEvent>>
}

impl RviServiceHandler {
    pub fn new(s: Sender<MessageEvent>) -> RviServiceHandler {
        RviServiceHandler {
            sender: Mutex::new(s)
        }
    }

    fn push_message_event(&self, e: MessageEvent) {
        self.sender.lock().unwrap().send(e).unwrap();
    }

    fn handle_message_params<D>(&self, b: &String)
        where D: Decodable + Clone + ToMessageEvent {
        match json::decode::<jsonrpc::Request<MessageParams<D>>>(&b) {
            Ok(p) => {
                self.push_message_event(
                    p.params.parameters[0].clone().to_event(
                        p.params.service_name,
                        p.id));
            },
            _ => {}
        }
    }

    fn handle_message(&self, b: &String) {
        // TODO: Parse JSON-RPC, without multiple matches on parameters
        // TODO: Avoid clone(), especially ChunkParams.msg
        self.handle_message_params::<NotifyParams>(b);
        self.handle_message_params::<StartParams>(b);
        self.handle_message_params::<ChunkParams>(b);
        self.handle_message_params::<FinishParams>(b);
    }
}

impl Handler for RviServiceHandler {
    fn handle(&self, mut req: Request, resp: Response) {
        let mut rbody = String::new();
        req.read_to_string(&mut rbody).unwrap();
        println!(">>> Received Message: {}", rbody);
        self.handle_message(&rbody);

        // TODO: Respond with JSON-RPC id and result from request
        let mut resp = resp.start().unwrap();
        resp.write_all(b"").unwrap();
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
        println!("<<< Send Message: {}", json_body);
        let mut resp = self.client.post(self.rvi_url.clone())
            .body(&json_body)
            .send()
            .unwrap();

        // TODO: Handle JSON-RPC response, match id
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
        let addr = (
            &*self.edge_url.host().unwrap().to_string(),
            self.edge_url.port().unwrap());
        let srv = Server::http(addr).unwrap();
        let _ = srv.handle(h);
    }
}
