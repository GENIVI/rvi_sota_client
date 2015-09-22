// TODO: Maybe make this a impl for rvi_url?

use std::io::Read;
use hyper::Client;
use rustc_serialize::{json, Encodable};

use jsonrpc;
use rvi::message::RVIMessage;

pub fn send<E: Encodable>(url: &str, b: &E) -> Result<String, String> {
    let client = Client::new();

    let json_body = match json::encode(b) {
        Ok(val) => val,
        Err(e) => { return Err(format!("{}", e)); }
    };

    debug!("<<< Sent Message: {}", json_body);

    let mut resp = match client.post(url).body(&json_body).send() {
        Ok(val) => val,
        Err(e) => { return Err(format!("{}", e)); }
    };

    let mut rbody = String::new();

    match resp.read_to_string(&mut rbody) {
        Ok(..) => {},
        Err(e) => { return Err(format!("{}", e)); }
    };

    debug!(">>> Received Response: {}", rbody);

    return Ok(rbody);
}

pub fn send_message<E: Encodable>(url: &str, b: E, addr: &str) -> Result<String, String> {
    let mut message = RVIMessage::<E>::new(addr, Vec::new(), 90);
    message.parameters.push(b);
    let json_rpc = jsonrpc::Request::new("message", message);
    return send(url, &json_rpc);
}
