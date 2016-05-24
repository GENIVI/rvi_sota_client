use rustc_serialize::{json, Decodable, Encodable};
use std::{env, thread};
use std::io::Read;
use std::sync::{Arc, Mutex, mpsc};
use std::sync::mpsc::{Sender, Receiver};
use hyper::status::StatusCode;
use hyper::server::{Handler, Server, Request, Response};

use super::gateway::{Gateway, Interpret};
use datatype::{Error, Event};


pub struct Http<C, E> {
    irx: Arc<Mutex<Receiver<Interpret<C, E>>>>,
}

impl<C, E> Gateway<C, E> for Http<C, E>
    where C: Send + Decodable + 'static,
          E: Send + Encodable + 'static
{
    fn new() -> Http<C, E> {
        let (itx, irx): (Sender<Interpret<C, E>>, Receiver<Interpret<C, E>>) = mpsc::channel();
        let handler = HttpHandler { itx: Arc::new(Mutex::new(itx)) };
        let addr    = env::var("OTA_PLUS_CLIENT_HTTP_ADDR")
                         .unwrap_or("127.0.0.1:8888".to_string());

        thread::spawn(move || {
            Server::http(&addr as &str).unwrap().handle(handler).unwrap();
        });

        Http { irx: Arc::new(Mutex::new(irx)) }
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


pub struct HttpHandler<C: Send + Decodable, E: Send + Encodable> {
    itx: Arc<Mutex<Sender<Interpret<C, E>>>>,
}

impl<C, E> Handler for HttpHandler<C, E>
    where C: Send + Decodable,
          E: Send + Encodable
{
    fn handle(&self, req: Request, resp: Response) {
        worker(self, req, resp).unwrap_or_else(|err| {
            error!("error handling request: {}", err);
        });

        fn worker<C, E>(handler: &HttpHandler<C, E>,
                        mut req: Request,
                        mut resp: Response)
                        -> Result<(), Error>
            where C: Send + Decodable,
                  E: Send + Encodable
        {
            // return 500 response on error
            *resp.status_mut() = StatusCode::InternalServerError;

            let mut req_body = String::new();
            let _: usize     = try!(req.read_to_string(&mut req_body));
            let c: C         = try!(json::decode(&req_body));

            let (etx, erx): (Sender<E>, Receiver<E>) = mpsc::channel();
            debug!("sending request body: {}", req_body);
            handler.itx.lock().unwrap().send(Interpret {
                cmd: c,
                etx: Some(Arc::new(Mutex::new(etx))),
            }).unwrap();

            match erx.recv() {
                Ok(e) => {
                    let resp_body = try!(json::encode(&e));
                    *resp.status_mut() = StatusCode::Ok;
                    debug!("sending response body: {}", resp_body);
                    resp.send(resp_body.as_bytes()).unwrap();
                }
                Err(err) => {
                    error!("error forwarding request: {}", err);
                    let ev = json::encode(&Event::Error(format!("{}", err))).unwrap();
                    resp.send(ev.as_bytes()).unwrap();
                }
            }

            Ok(())
        }
    }
}


#[cfg(test)]
mod tests {
    use hyper::Client;
    use rustc_serialize::json;
    use std::thread;
    use std::io::Read;
    use std::sync::mpsc::{channel, Sender, Receiver};

    use super::*;
    use super::super::gateway::Gateway;
    use super::super::super::datatype::{Command, Event};
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

        let mut threads = vec![];
        for id in 0..10 {
            threads.push(thread::spawn(move || {
                let client = Client::new();
                let cmd    = Command::AcceptUpdate(format!("{}", id));

                let req_body = json::encode(&cmd).unwrap();
                let mut resp = client.post("http://127.0.0.1:8888/")
                                     .body(&req_body)
                                     .send()
                                     .unwrap();

                let mut resp_body = String::new();
                resp.read_to_string(&mut resp_body).unwrap();
                let ev: Event = json::decode(&resp_body).unwrap();
                assert_eq!(ev, Event::Error(format!("{}", id)));
            }));
        }

        // wait for all threads to finish
        for t in threads {
            let _ = t.join();
        }
    }
}
