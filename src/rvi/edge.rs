use std::sync::mpsc::Sender;

use hyper::Server;
use hyper::server::{Handler, Request};

use jsonrpc;
use rustc_serialize::json;

use rvi::{send, RVIHandler};
use rvi::message::{RegisterServiceRequest, RegisterServiceResponse};

#[derive(Clone)]
pub struct Service {
    pub name: String,
    pub addr: String
}

pub struct ServiceEdge {
    rvi_url: String,
    edge_url: String,
    sender: Sender<Vec<Service>>
}

impl ServiceEdge {
    pub fn new(r: String,
               e: String,
               s: Sender<Vec<Service>>) -> ServiceEdge {
        ServiceEdge {
            rvi_url: r,
            edge_url: e,
            sender: s
        }
    }

    pub fn register_service(&self, s: &str) -> String {
        let json_rpc = jsonrpc::Request::new(
            "register_service",
            RegisterServiceRequest {
                network_address: self.edge_url.to_string(),
                service: s.to_string()
            });

        let resp = send::send(&self.rvi_url, &json_rpc)
            .map_err(|e| error!("Couldn't send registration to RVI\n{}", e))
            .and_then(|r| json::decode::<jsonrpc::OkResponse<RegisterServiceResponse>>(&r)
                      .map_err(|e| error!("Couldn't parse response when registering in RVI\n{}", e)))
            .unwrap();

        resp.result
            .expect("Didn't get full service name when registering")
            .service
    }

    fn register(&self, svcs: &Vec<&str>) -> Vec<Service> {
        svcs.iter().map(|s: &&str| {
            Service {
                name: s.to_string(),
                addr: self.register_service(s)
            }
        }).collect()
    }

    pub fn start<H: 'static>(&self, h: H, s: Vec<&str>)
        where H: Handler + RVIHandler {
        let mut handler = h;

        let services = self.register(&s);
        self.sender.send(services.clone())
            .map_err(|e| error!("Couldn't send registration to RVI\n{}", e))
            .unwrap();

        handler.register(services);

        let url: &str = &self.edge_url;
        Server::http(url)
            .map_err(|e| error!("Couldn't start server\n{}", e))
            .and_then(|srv| {
                info!("Ready to accept connections.");
                srv.handle(handler).unwrap();
                Ok(()) })
            .unwrap()
    }
}
