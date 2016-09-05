use hyper::{Decoder, Encoder, Next, StatusCode};
use hyper::header::{ContentLength, ContentType};
use hyper::mime::{Attr, Mime, TopLevel, SubLevel, Value};
use hyper::net::Transport;
use hyper::server::{Handler, Request as HyperRequest, Response as HyperResponse};
use std::{mem, io};
use std::io::{ErrorKind, Write};
use std::time::Duration;


/// An HTTP server handles the incoming headers and request body as well as the
/// setting the response status and body. Other concerns regarding the asynchronous
/// event loop handlers for writing to buffers are abstracted away.
pub trait Server<T: Transport>: Send {
    fn headers(&mut self, req: HyperRequest<T>);
    fn request(&mut self, body: Vec<u8>);
    fn response(&mut self) -> (StatusCode, Option<Vec<u8>>);
}


/// This implements the `hyper::server::Handler` trait so that it can be used
/// to handle incoming HTTP connections with `hyper::server::Server`.
pub struct ServerHandler<T: Transport> {
    server:    Box<Server<T>>,
    req_body:  Vec<u8>,
    resp_body: Vec<u8>,
    written:   usize
}

impl<T: Transport> ServerHandler<T> {
    /// Instantiate a new `ServerHandler` by passing a `Box<Server<T>` reference.
    pub fn new(server: Box<Server<T>>) -> Self {
        ServerHandler {
            server:    server,
            req_body:  Vec::new(),
            resp_body: Vec::new(),
            written:   0
        }
    }
}

impl<T: Transport> Handler<T> for ServerHandler<T> {
    fn on_request(&mut self, req: HyperRequest<T>) -> Next {
        info!("on_request: {} {}", req.method(), req.uri());
        self.server.headers(req);
        Next::read()
    }

    fn on_request_readable(&mut self, transport: &mut Decoder<T>) -> Next {
        match io::copy(transport, &mut self.req_body) {
            Ok(0) => {
                debug!("on_request_readable bytes read: {}", self.req_body.len());
                self.server.request(mem::replace(&mut self.req_body, Vec::new()));
                Next::write().timeout(Duration::from_secs(20))
            }

            Ok(n) => {
                trace!("{} more request bytes read", n);
                Next::read()
            }

            Err(ref err) if err.kind() == ErrorKind::WouldBlock => {
                trace!("retry on_request_readable");
                Next::read()
            }

            Err(err) => {
                error!("unable to read request body: {}", err);
                Next::remove()
            }
        }
    }

    fn on_response(&mut self, resp: &mut HyperResponse) -> Next {
        let (status, body) = self.server.response();
        resp.set_status(status);
        info!("on_response: status {}", resp.status());

        let mut headers = resp.headers_mut();
        headers.set(ContentType(Mime(TopLevel::Application, SubLevel::Json,
                                     vec![(Attr::Charset, Value::Utf8)])));
        body.map_or_else(Next::end, |body| {
            headers.set(ContentLength(body.len() as u64));
            self.resp_body = body;
            Next::write()
        })
    }

    fn on_response_writable(&mut self, transport: &mut Encoder<T>) -> Next {
        match transport.write(&self.resp_body[self.written..]) {
            Ok(0) => {
                debug!("{} bytes written to response body", self.written);
                Next::end()
            }

            Ok(n) => {
                self.written += n;
                trace!("{} bytes written to response body", n);
                Next::write()
            }

            Err(ref err) if err.kind() == ErrorKind::WouldBlock => {
                trace!("retry on_response_writable");
                Next::write()
            }

            Err(err) => {
                error!("unable to write response body: {}", err);
                Next::remove()
            }
        }
    }
}
