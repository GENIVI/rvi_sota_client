use std::io::Read;
use hyper::Client;
use url::Url;
use rustc_serialize::{json, Encodable};

pub fn send<E: Encodable>(rvi_url: Url, b: &E) {
    let client = Client::new();
    let json_body = json::encode(b).unwrap();

    debug!("<<< Sent Message: {}", json_body);
    let mut resp = client.post(rvi_url.clone())
        .body(&json_body)
        .send()
        .unwrap();

    let mut rbody = String::new();
    resp.read_to_string(&mut rbody).unwrap();
    debug!(">>> Received Response: {}", rbody);
}
