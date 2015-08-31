use jsonrpc;

use hyper::Server;
use hyper::server::{Handler, Request};
use url::Url;
use rustc_serialize::Encodable;

use rvi::message::*;
use rvi::send::send;

use rvi::service_handler::RviServiceHandler;

#[derive(RustcEncodable)]
struct RegisterServiceParams {
    network_address: String,
    service: String
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
