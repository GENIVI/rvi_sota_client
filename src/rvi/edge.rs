//! Implements the RVI facing webservice.

use hyper::Server;
use hyper::server::{Handler, Request};

use jsonrpc;
use rustc_serialize::json;

use rvi::send;
use rvi::message::{RegisterServiceRequest, RegisterServiceResponse};

/// Encodes the service edge of the webservice.
pub struct ServiceEdge {
    /// The full URL where RVI can be reached.
    rvi_url: String,
    /// The `host:port` combination where the edge should bind and listen for incoming RVI
    /// messages.
    edge_url: String
}

impl ServiceEdge {
    /// Create a new service edge.
    ///
    /// # Arguments
    /// * `r`: The full URL where RVI can be reached.
    /// * `e`: The `host:port` combination where the edge should bind.
    /// * `s`: A sender to communicate back the service URLs.
    pub fn new(r: String, e: String) -> ServiceEdge {
        ServiceEdge {
            rvi_url: r,
            edge_url: e
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

        let resp = send(&self.rvi_url, &json_rpc)
            .map_err(|e| error!("Couldn't send registration to RVI\n{}", e))
            .and_then(|r| json::decode::<jsonrpc::OkResponse<RegisterServiceResponse>>(&r)
                      .map_err(|e| error!("Couldn't parse response when registering in RVI\n{}", e)))
            .unwrap();

        resp.result
            .expect("Didn't get full service name when registering")
            .service
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
    pub fn start<H: 'static>(&self, h: H)
        where H: Handler {
        let url: &str = &self.edge_url;
        Server::http(url).and_then(|srv| {
            info!("Ready to accept connections.");
            srv.handle(h) })
            .map_err(|e| error!("Couldn't start server\n{}", e))
            .unwrap();
    }
}
