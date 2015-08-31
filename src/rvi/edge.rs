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

        let rbody = match send::send(&self.rvi_url, &json_rpc) {
            Ok(val) => val,
            Err(e) => {
                error!("Couldn't send registration to RVI\n{}", e);
                panic!("Couldn't register in RVI!");
            }
        };
        let response = match json::decode::<jsonrpc::OkResponse<RegisterServiceResponse>>(&rbody) {
            Ok(val) => val,
            Err(e) => {
                error!("Couldn't parse response when registering in RVI");
                error!("{}", e);
                panic!("Couldn't register in RVI!");
            }
        };
        match response.result {
            Some(r) => { return r.service; },
            None => {
                error!("Didn't get full service name when registering");
                panic!("Couldn't register in RVI!");
            }
        }
    }

    fn register(&self, s: &Vec<&str>) -> Vec<Service> {
        let mut services = Vec::new();

        for service in s {
            let registered_service = Service {
                name: service.to_string(),
                addr: self.register_service(service)
            };
            services.push(registered_service);
        }

        return services;
    }

    pub fn start<H: 'static>(&self, h: H, s: Vec<&str>)
        where H: Handler, H: RVIHandler {
        let mut handler = h;

        let services = self.register(&s);
        match self.sender.send(services.clone()) {
            Ok(..) => {},
            Err(e) => {
                error!("Couldn't send registration to RVI\n{}", e);
                panic!("Couldn't register in RVI!");
            }
        }

        handler.register(services);

        let url: &str = &self.edge_url;
        match Server::http(url) {
            Ok(srv) => {
                info!("Ready to accept connections.");
                srv.handle(handler).unwrap();
            },
            Err(msg) => {
                error!("Couldn't start server. {}", msg);
                panic!(msg);
            }
        }
    }
}
