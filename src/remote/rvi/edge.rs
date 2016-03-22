//! Implements the RVI facing webservice.

use std::io::{Read, Write};
use std::thread;
use hyper::Server;
use hyper::server::{Handler, Request, Response};
use rustc_serialize::json;
use rustc_serialize::json::Json;

use remote::jsonrpc;
use remote::jsonrpc::{OkResponse, ErrResponse};

use remote::rvi::send;
use remote::rvi::message::{RegisterServiceRequest, RegisterServiceResponse};

pub trait ServiceHandler: Sync + Send {
    fn handle_service(&self, id: u64, service: &str, message: &str)
        -> Result<OkResponse<i32>, ErrResponse>;
    fn register_services<F: Fn(&str) -> String>(&self, reg: F);
}

/// Encodes the service edge of the webservice.
pub struct ServiceEdge<H: ServiceHandler + 'static> {
    /// The full URL where RVI can be reached.
    rvi_url: String,
    /// The `host:port` to bind and listen for incoming RVI messages.
    edge_url: String,
    hdlr: H
}

impl<H: ServiceHandler + 'static> ServiceEdge<H> {
    /// Create a new service edge.
    ///
    /// # Arguments
    /// * `r`: The full URL where RVI can be reached.
    /// * `e`: The `host:port` combination where the edge should bind.
    /// * `s`: A sender to communicate back the service URLs.
    pub fn new(r: String, e: String, h: H) -> ServiceEdge<H> {
        ServiceEdge {
            rvi_url: r,
            edge_url: e,
            hdlr: h
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
    pub fn start(self) {
        let url = self.edge_url.clone();
        self.hdlr.register_services(|s| self.register_service(s));
        thread::spawn(move || {
            Server::http(&*url).and_then(|srv| {
                info!("Ready to accept connections.");
                srv.handle(self) })
                .map_err(|e| error!("Couldn't start server\n{}", e))
                .unwrap()
        });
    }

    /// Try to parse the type of a message and forward it to the appropriate message handler.
    /// Returns the result of the message handling or a `jsonrpc` result indicating a parser error.
    ///
    /// Needs to be extended to support new services.
    ///
    /// # Arguments
    /// * `message`: The message that will be parsed.
    fn handle_message(&self, message: &str)
        -> Result<OkResponse<i32>, ErrResponse> {

        let data = try!(
            Json::from_str(message)
                .map_err(|_| ErrResponse::parse_error()));
        let obj = try!(
            data.as_object().ok_or(ErrResponse::parse_error()));
        let rpc_id = try!(
            obj.get("id").and_then(|x| x.as_u64())
                .ok_or(ErrResponse::parse_error()));

        let method = try!(
            obj.get("method").and_then(|x| x.as_string())
                .ok_or(ErrResponse::invalid_request(rpc_id)));

        if method == "services_available" {
            Ok(OkResponse::new(rpc_id, None))
        }
        else if method != "message" {
            Err(ErrResponse::method_not_found(rpc_id))
        } else {
            let service = try!(obj.get("params")
                               .and_then(|x| x.as_object())
                               .and_then(|x| x.get("service_name"))
                               .and_then(|x| x.as_string())
                               .ok_or(ErrResponse::invalid_request(rpc_id)));

            self.hdlr.handle_service(rpc_id, service, message)
        }
    }
}

impl<H: ServiceHandler + 'static> Handler for ServiceEdge<H> {
    fn handle(&self, mut req: Request, resp: Response) {
        let mut rbody = String::new();
        try_or!(req.read_to_string(&mut rbody), return);
        debug!(">>> Received Message: {}", rbody);
        let mut resp = try_or!(resp.start(), return);

        macro_rules! send_response {
            ($rtype:ty, $resp:ident) => {
                match json::encode::<$rtype>(&$resp) {
                    Ok(decoded_msg) => {
                        try_or!(resp.write_all(decoded_msg.as_bytes()), return);
                        debug!("<<< Sent Response: {}", decoded_msg);
                    },
                    Err(p) => { error!("{}", p); }
                }
            };
        }

        match self.handle_message(&rbody) {
            Ok(msg) => send_response!(OkResponse<i32>, msg),
            Err(msg) => send_response!(ErrResponse, msg)
        }

        try_or!(resp.end(), return);
    }
}
