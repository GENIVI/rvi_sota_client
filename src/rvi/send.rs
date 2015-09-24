// TODO: Maybe make this a impl for rvi_url?

use std::io::Read;
use hyper::Client;
use rustc_serialize::{json, Encodable};

use jsonrpc;
use rvi::message::RVIMessage;

pub fn send<E: Encodable>(url: &str, b: &E) -> Result<String, String> {
    let client = Client::new();

    let mut resp = try!(json::encode(b)
        .map_err(|e| format!("{}", e))
        .and_then(|j| {
            debug!("<<< Sent Message: {}", j);
            client.post(url).body(&j).send()
                .map_err(|e| format!("{}", e))
        }));

    let mut rbody = String::new();
    try!(resp.read_to_string(&mut rbody)
         .map_err(|e| format!("{}", e)));
    debug!(">>> Received Response: {}", rbody);
    Ok(rbody)
}

pub fn send_message<E: Encodable>(url: &str, b: E, addr: &str) -> Result<String, String> {
    let mut params = Vec::new();
    params.push(b);
    let message = RVIMessage::<E>::new(addr, params, 90);
    let json_rpc = jsonrpc::Request::new("message", message);
    send(url, &json_rpc)
}
