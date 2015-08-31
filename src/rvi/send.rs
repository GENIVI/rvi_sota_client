use std::io::Read;
use hyper::Client;
use url::Url;
use rustc_serialize::{json, Encodable};
use rvi::message::{Message, InitiateParams};
use jsonrpc;

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

pub fn initiate_download(rvi_url: Url, package: String, id: u32) {
    let mut message = Message::<InitiateParams> {
        service_name: "genivi.org/backend/sota/initiate_download".to_string(),
        parameters: Vec::new()
    };

    let params = InitiateParams{
        id: id,
        package: package
    };

    message.parameters.push(params);
    let json_rpc = jsonrpc::Request::new("message", message);
    send(rvi_url.clone(), &json_rpc);
}
