use hyper::{Decoder, Encoder, Next, StatusCode};
use hyper::net::HttpStream;
use hyper::server::{Handler, Server, Request, Response};
use rustc_serialize::{json, Decodable, Encodable};
use std::{env, thread};
use std::fmt::Debug;
use std::io::{ErrorKind, Read, Write};
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
    itx:  Arc<Mutex<Sender<Interpret<C, E>>>>,
    erx:  Option<Receiver<E>>,
    body: Option<Vec<u8>>,
}

impl<C, E> HttpHandler<C, E>
    where C: Decodable + Send + Clone + Debug,
          E: Encodable + Send + Clone
{
    fn new(itx: Arc<Mutex<Sender<Interpret<C, E>>>>) -> HttpHandler<C, E> {
        HttpHandler { itx:  itx, erx:  None, body: None }
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
        info!("on_request_readable");

        let mut data = Vec::new();
        match transport.read_to_end(&mut data) {
            Ok(_) => match String::from_utf8(data) {
                Ok(body) => match json::decode::<C>(&body) {
                    Ok(c) => {
                        info!("on_request_readable: decoded command: {:?}", c);

                        let (etx, erx): (Sender<E>, Receiver<E>) = mpsc::channel();
                        self.erx = Some(erx);
                        self.itx.lock().unwrap().send(Interpret {
                            cmd: c,
                            etx: Some(Arc::new(Mutex::new(etx))),
                        }).unwrap();

                        Next::write().timeout(Duration::from_secs(10))
                    }

                    Err(err) => {
                        error!("on_request_readable decode json: {}", err);
                        Next::remove()
                    }
                },

                Err(err) => {
                    error!("on_request_readable parse string: {}", err);
                    Next::remove()
                }
            },

            Err(err) => match err.kind() {
                ErrorKind::WouldBlock => Next::read(),
                _                     => {
                    error!("on_request_readable read_to_end: {}", err);
                    Next::remove()
                }
            }
        }
    }

    fn on_response(&mut self, resp: &mut Response) -> Next {
        info!("on_response: status {}", resp.status());

        match self.erx {
            Some(ref rx) => match rx.recv() {
                Ok(e) => match json::encode(&e) {
                    Ok(body) => {
                        resp.set_status(StatusCode::Ok);
                        self.body = Some(body.into_bytes());
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
            },

            None => panic!("expected Some receiver")
        }
    }

    fn on_response_writable(&mut self, transport: &mut Encoder<HttpStream>) -> Next {
        info!("on_response_writable");

        match self.body {
            Some(ref body) => match transport.write_all(body) {
                Ok(_)    => Next::end(),

                Err(err) => match err.kind() {
                    ErrorKind::WouldBlock => Next::write(),
                    _                     => {
                        error!("unable to write body: {}", err);
                        Next::remove()
                    }
                }
            },

            None => panic!("on_response_writable called on empty body")
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
