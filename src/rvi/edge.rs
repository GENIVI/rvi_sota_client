//! Implements the RVI facing webservice.

use std::sync::mpsc::Sender;

use hyper::Server;
use hyper::server::{Handler, Request};

use jsonrpc;
use rustc_serialize::json;

use rvi::{send, RVIHandler};
use rvi::message::{RegisterServiceRequest, RegisterServiceResponse};

/// Encodes a registered service that this device provides.
#[derive(Clone)]
pub struct Service {
    /// The last part of the URL, identifying the service. Can be used as internal identifier.
    pub name: String,
    /// The URL, that RVI provides for this service.
    pub addr: String
}

/// Encodes the service edge of the webservice.
pub struct ServiceEdge {
    /// The full URL where RVI can be reached.
    rvi_url: String,
    /// The `host:port` combination where the edge should bind and listen for incoming RVI
    /// messages.
    edge_url: String,
    /// A sender to communicate back the service URLs.
    sender: Sender<Vec<Service>>
}

impl ServiceEdge {
    /// Create a new service edge.
    ///
    /// # Arguments
    /// * `r`: The full URL where RVI can be reached.
    /// * `e`: The `host:port` combination where the edge should bind.
    /// * `s`: A sender to communicate back the service URLs.
    pub fn new(r: String,
               e: String,
               s: Sender<Vec<Service>>) -> ServiceEdge {
        ServiceEdge {
            rvi_url: r,
            edge_url: e,
            sender: s
        }
    }

    /// Register a service. Returns the full service URL as provided by RVI. Panics if the
    /// registration in RVI failed. This can be handled by starting the RVI edge in a separate
    /// thread.
    ///
    /// # Arguments
    /// * `s`: The service to register. Will get prepended with the device identifier by RVI.
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

    /// Helper function to register multiple services. Returns a `Vector` of `Service`s.
    ///
    /// # Arguments
    /// * `svcs`: Pointer to a `Vector` of service strings, that should be registered.
    fn register(&self, svcs: &Vec<&str>) -> Vec<Service> {
        svcs.iter().map(|s: &&str| {
            Service {
                name: s.to_string(),
                addr: self.register_service(s)
            }
        }).collect()
    }

    /// Starts the service edge.
    ///
    /// It binds on the provided `host:port` combination, registers all services and then waits for
    /// incoming RVI messages. On incoming messages it forks another thread and passes the message
    /// to the provided `Handler`. For details about how to implement a `Handler` see the
    /// [`hyper`](../../hyper/index.html) documentation and the [reference
    /// implementation](../handler/index.html).
    ///
    /// Panics if it can't reach or register in RVI.
    ///
    /// # Arguments
    /// * `h`: The `Handler` all messages are passed to.
    /// * `s`: A `Vector` of service strings to register in RVI.
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
