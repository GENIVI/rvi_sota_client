use hyper::StatusCode;
use hyper::net::{HttpStream, Transport};
use hyper::server::{Server as HyperServer, Request as HyperRequest};
use rustc_serialize::json;
use rustc_serialize::json::Json;
use std::{mem, str};
use std::net::ToSocketAddrs;

use datatype::{RpcRequest, RpcOk, RpcErr, Url};
use http::{Server, ServerHandler};
use super::services::Services;


/// The HTTP server endpoint for `RVI` client communication.
pub struct Edge {
    rvi_edge: Url,
    services: Services,
}

impl Edge {
    /// Create a new `Edge` by registering each `RVI` service.
    pub fn new(mut services: Services, rvi_edge: Url, rvi_client: Url) -> Self {
        services.register_services(|service| {
            let req = RpcRequest::new("register_service", RegisterServiceRequest {
                network_address: rvi_edge.to_string(),
                service:         service.to_string(),
            });
            let resp = req.send(rvi_client.clone())
                .unwrap_or_else(|err| panic!("RegisterServiceRequest failed: {}", err));
            let rpc_ok = json::decode::<RpcOk<RegisterServiceResponse>>(&resp)
                .unwrap_or_else(|err| panic!("couldn't decode RegisterServiceResponse: {}", err));
            rpc_ok.result.expect("expected rpc_ok result").service
        });

        Edge { rvi_edge: rvi_edge, services: services }
    }

    /// Start the HTTP server listening for incoming RVI client connections.
    pub fn start(&mut self) {
        let mut addrs = self.rvi_edge.to_socket_addrs()
            .unwrap_or_else(|err| panic!("couldn't parse edge url: {}", err));
        let server = HyperServer::http(&addrs.next().expect("no SocketAddr found"))
            .unwrap_or_else(|err| panic!("couldn't start rvi edge server: {}", err));
        let (addr, server) = server.handle(move |_| EdgeHandler::new(self.services.clone())).unwrap();
        info!("RVI server edge listening at http://{}.", addr);
        server.run();
    }
}


#[derive(RustcEncodable)]
struct RegisterServiceRequest {
    pub network_address: String,
    pub service:         String,
}

#[derive(RustcDecodable)]
struct RegisterServiceResponse {
    pub service: String,
    pub status:  i32,
}



struct EdgeHandler {
    services:  Services,
    resp_code: StatusCode,
    resp_body: Option<Vec<u8>>
}

impl EdgeHandler {
    fn new(services: Services) -> ServerHandler<HttpStream> {
        ServerHandler::new(Box::new(EdgeHandler {
            services:  services,
            resp_code: StatusCode::InternalServerError,
            resp_body: None,
        }))
    }
}

impl<T: Transport> Server<T> for EdgeHandler {
    fn headers(&mut self, _: HyperRequest<T>) {}

    fn request(&mut self, body: Vec<u8>) {
        let outcome = || -> Result<RpcOk<i32>, RpcErr> {
            let text   = try!(str::from_utf8(&body).map_err(|err| RpcErr::parse_error(err.to_string())));
            let data   = try!(Json::from_str(text).map_err(|err| RpcErr::parse_error(err.to_string())));
            let object = try!(data.as_object().ok_or(RpcErr::parse_error("not an object".to_string())));
            let id     = try!(object.get("id").and_then(|x| x.as_u64())
                              .ok_or(RpcErr::parse_error("expected id".to_string())));
            let method = try!(object.get("method").and_then(|x| x.as_string())
                              .ok_or(RpcErr::invalid_request(id, "expected method".to_string())));

            match method {
                "services_available" => Ok(RpcOk::new(id, None)),

                "message" => {
                    let params  = try!(object.get("params").and_then(|p| p.as_object())
                                       .ok_or(RpcErr::invalid_request(id, "expected params".to_string())));
                    let service = try!(params.get("service_name").and_then(|s| s.as_string())
                                       .ok_or(RpcErr::invalid_request(id, "expected params.service_name".to_string())));
                    self.services.handle_service(service, id, text)
                }

                _ => Err(RpcErr::method_not_found(id, format!("unknown method: {}", method)))
            }
        }();

        match outcome {
            Ok(msg)  => {
                let body = json::encode::<RpcOk<i32>>(&msg).expect("couldn't encode RpcOk response");
                self.resp_code = StatusCode::Ok;
                self.resp_body = Some(body.into_bytes());
            }

            Err(err) => {
                let body = json::encode::<RpcErr>(&err).expect("couldn't encode RpcErr response");
                self.resp_code = StatusCode::BadRequest;
                self.resp_body = Some(body.into_bytes());
            }
        }
    }

    fn response(&mut self) -> (StatusCode, Option<Vec<u8>>) {
        (self.resp_code, mem::replace(&mut self.resp_body, None))
    }
}
