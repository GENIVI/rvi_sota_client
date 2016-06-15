use hyper::{Decoder, Encoder, Next, StatusCode};
use hyper::header::ContentType;
use hyper::mime::{Attr, Mime, TopLevel, SubLevel, Value};
use hyper::net::HttpStream;
use hyper::server::{Handler, Server, Request, Response};
use rustc_serialize::{json, Decodable, Encodable};
use std::{env, io, mem, thread};
use std::fmt::Debug;
use std::io::{ErrorKind, Write};
use std::sync::{Arc, Mutex, mpsc};
use std::sync::mpsc::{Sender, Receiver};
use std::time::Duration;

use super::gateway::{Gateway, Interpret};


pub struct Http<C: Clone, E: Clone> {
    irx: Arc<Mutex<Receiver<Interpret<C, E>>>>,
}

impl<C, E> Gateway<C, E> for Http<C, E>
    where C: Decodable + Send + Clone + Debug + 'static,
          E: Encodable + Send + Clone + 'static
{
    fn new() -> Http<C, E> {
        let (itx, irx): (Sender<Interpret<C, E>>, Receiver<Interpret<C, E>>) = mpsc::channel();
        let itx = Arc::new(Mutex::new(itx));
        let irx = Arc::new(Mutex::new(irx));

        let addr = env::var("OTA_PLUS_CLIENT_HTTP_ADDR").unwrap_or("127.0.0.1:8888".to_string());
        let server = Server::http(&addr.parse().unwrap()).unwrap();
        let (addr, server) = server.handle(move |_| HttpHandler::new(itx.clone())).unwrap();

        thread::spawn(move || { server.run() });
        info!("Listening on http://{}", addr);

        Http { irx: irx }
    }

    fn next(&self) -> Option<Interpret<C, E>> {
        match self.irx.lock().unwrap().recv() {
            Ok(i)    => Some(i),
            Err(err) => {
                error!("Error forwarding request: {}", err);
                None
            }
        }
    }
}


pub struct HttpHandler<C, E>
    where C: Decodable + Send + Clone + Debug,
          E: Encodable + Send + Clone
{
    itx:       Arc<Mutex<Sender<Interpret<C, E>>>>,
    erx:       Option<Receiver<E>>,
    req_body:  Vec<u8>,
    resp_body: Option<Vec<u8>>,
    written:   usize,
}

impl<C, E> HttpHandler<C, E>
    where C: Decodable + Send + Clone + Debug,
          E: Encodable + Send + Clone
{
    fn new(itx: Arc<Mutex<Sender<Interpret<C, E>>>>) -> HttpHandler<C, E> {
        HttpHandler {
            itx:       itx,
            erx:       None,
            req_body:  Vec::new(),
            resp_body: None,
            written:   0
        }
    }

    fn decode_request(&mut self) -> Next {
        let body = mem::replace(&mut self.req_body, Vec::new());

        match String::from_utf8(body) {
            Ok(body) => match json::decode::<C>(&body) {
                Ok(c) => {
                    info!("on_request_readable: decoded command: {:?}", c);

                    let (etx, erx): (Sender<E>, Receiver<E>) = mpsc::channel();
                    self.erx = Some(erx);
                    self.itx.lock().unwrap().send(Interpret {
                        cmd: c,
                        etx: Some(Arc::new(Mutex::new(etx))),
                    }).unwrap();

                    Next::write().timeout(Duration::from_secs(20))
                }

                Err(err) => {
                    error!("decode_request: parse json: {}", err);
                    Next::remove()
                }
            },

            Err(err) => {
                error!("decode_request: parse string: {}", err);
                Next::remove()
            }
        }
    }
}

impl<C, E> Handler<HttpStream> for HttpHandler<C, E>
    where C: Send + Decodable + Clone + Debug,
          E: Send + Encodable + Clone
{
    fn on_request(&mut self, req: Request) -> Next {
        info!("on_request: {} {}", req.method(), req.uri());
        Next::read()
    }

    fn on_request_readable(&mut self, transport: &mut Decoder<HttpStream>) -> Next {
        match io::copy(transport, &mut self.req_body) {
            Ok(0) => {
                debug!("on_request_readable bytes read: {:?}", self.req_body.len());
                self.decode_request()
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

    fn on_response(&mut self, resp: &mut Response) -> Next {
        info!("on_response: status {}", resp.status());
        let rx = self.erx.as_ref().expect("Some receiver expected");

        match rx.recv() {
            Ok(e) => match json::encode(&e) {
                Ok(body) => {
                    resp.set_status(StatusCode::Ok);
                    resp.headers_mut().set(ContentType(Mime(TopLevel::Application, SubLevel::Json,
                                                            vec![(Attr::Charset, Value::Utf8)])));
                    self.resp_body = Some(body.into_bytes());
                    Next::write()
                }

                Err(err) => {
                    error!("on_response encoding json: {:?}", err);
                    resp.set_status(StatusCode::InternalServerError);
                    Next::end()
                }
            },

            Err(err) => {
                error!("on_response receiver: {}", err);
                resp.set_status(StatusCode::InternalServerError);
                Next::end()
            }
        }
    }

    fn on_response_writable(&mut self, transport: &mut Encoder<HttpStream>) -> Next {
        let body = self.resp_body.as_ref().expect("on_response_writable has empty body");

        match transport.write(&body[self.written..]) {
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


#[cfg(test)]
mod tests {
    use crossbeam;
    use rustc_serialize::json;
    use std::thread;
    use std::sync::mpsc::{channel, Sender, Receiver};

    use super::*;
    use super::super::gateway::Gateway;
    use super::super::super::datatype::{Auth, Command, Event, Method, Url};
    use super::super::super::http_client::{AuthClient, HttpClient, HttpRequest};
    use super::super::super::interpreter::Wrapped;


    #[test]
    fn multiple_connections() {
        let (_,   erx): (Sender<Event>,   Receiver<Event>)   = channel();
        let (wtx, wrx): (Sender<Wrapped>, Receiver<Wrapped>) = channel();
        Http::run(wtx, erx);

        thread::spawn(move || {
            loop {
                let w = wrx.recv().unwrap();
                match w.cmd {
                    Command::AcceptUpdate(id) => {
                        let ev = Event::Error(id);
                        match w.etx {
                            Some(etx) => etx.lock().unwrap().send(ev).unwrap(),
                            None      => panic!("expected transmitter"),
                        }
                    }
                    _ => panic!("expected AcceptUpdate"),
                }
            }
        });

        // wait for all scoped threads to complete
        crossbeam::scope(|scope| {
            for id in 0..10 {
                scope.spawn(move || {
                    let client   = AuthClient::new(Auth::None);
                    let cmd      = Command::AcceptUpdate(format!("{}", id));
                    let req_body = json::encode(&cmd).unwrap();

                    let req = HttpRequest {
                        method: Method::Post,
                        url:    Url::parse("http://127.0.0.1:8888").unwrap(),
                        body:   Some(req_body.into_bytes()),
                    };
                    let resp_rx = client.send_request(req);
                    let resp    = resp_rx.recv().unwrap().unwrap();

                    let text      = String::from_utf8(resp).unwrap();
                    let ev: Event = json::decode(&text).unwrap();
                    assert_eq!(ev, Event::Error(format!("{}", id)));
                });
            }
        });
    }
}
