use jsonrpc;
use jsonrpc::{OkResponse, ErrResponse};

use std::io::{Read, Write};
use std::sync::Mutex;
use std::sync::mpsc::Sender;

use std::vec::Vec;

use hyper::server::{Handler, Request, Response};
use url::Url;
use rustc_serialize::{json, Decodable};

use std::collections::HashMap;
use persistence::PackageFile;

use rvi::message::*;
use rvi::send::send;

pub struct RviServiceHandler {
    sender: Mutex<Sender<(String, u32)>>,
    pending: Mutex<HashMap<String, i32>>,
    transfers: Mutex<HashMap<u32, PackageFile>>,
    rvi_url: Url,
}

impl RviServiceHandler {
    pub fn new(s: Sender<(String, u32)>, u: Url) -> RviServiceHandler {
        RviServiceHandler {
            sender: Mutex::new(s),
            rvi_url: u,
            pending: Mutex::new(HashMap::new()),
            transfers: Mutex::new(HashMap::new()),
        }
    }

    fn push_notify(&self, pkg: String) {
        let mut transfers = self.transfers.lock().unwrap();

        // Find first free id to use
        let mut id: u32 = 1;
        while transfers.contains_key(&id) {
            id = id + 1;
        }

        // reserve the id
        let pfile = PackageFile::new(&pkg, 0, 0);
        transfers.insert(id, pfile);

        self.sender.lock().unwrap().send((pkg, id)).unwrap();
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
        macro_rules! handle_params {
            ($s:ident, $b:ident, $( $x:ty ), *) => {
                $(
                match $s.handle_message_params::<$x>($b) {
                    Some(result) => {return result;},
                    None => {}
                }
                )*
            }
        }

        handle_params!(self, b,
                       NotifyParams,
                       StartParams,
                       ChunkParams,
                       FinishParams);

        match json::decode::<jsonrpc::Request<Message<String>>>(b) {
            Ok(p) => { Err(ErrResponse::method_not_found(p.id)) },
            _     => { Err(ErrResponse::parse_error()) }
        }
    }

    fn send_ack(&self, ack: GenericAck) {
        macro_rules! send_specific_ack {
            ($x:ty, $p:ident) => {{
                let mut message = Message::<$x> {
                    service_name: "genivi.org/backend/sota/ack".to_string(),
                    parameters: Vec::new()
                };
                message.parameters.push($p);
                let json_rpc = jsonrpc::Request::new("message", message);
                send(self.rvi_url.clone(), &json_rpc);
            }};
        }

        match ack {
            GenericAck::Ack(p)   => { send_specific_ack!(AckParams, p); },
            GenericAck::Chunk(p) => { send_specific_ack!(AckChunkParams, p); }
        }
    }
}

impl Handler for RviServiceHandler {
    fn handle(&self, mut req: Request, resp: Response) {
        let mut rbody = String::new();
        req.read_to_string(&mut rbody).unwrap();
        debug!(">>> Received Message: {}", rbody);
        let mut resp = resp.start().unwrap();

        macro_rules! send_response {
            ($rtype:ty, $resp:ident) => {
                match json::encode::<$rtype>(&$resp) {
                    Ok(decoded_msg) => {
                        resp.write_all(decoded_msg.as_bytes()).unwrap();
                        debug!("<<< Sent Response: {}", decoded_msg);
                    },
                    Err(p) => { error!("!!! ERR: {}", p); }
                }
            };
        }

        match self.handle_message(&rbody) {
            Ok(msg) => { send_response!(OkResponse, msg) },
            Err(msg) => { send_response!(ErrResponse, msg) }
        }

        resp.end().unwrap();
    }
}
