use chan;
use chan::{Sender, Receiver};
use hyper::StatusCode;
use hyper::net::{HttpStream, Transport};
use hyper::server::{Server as HyperServer, Request as HyperRequest};
use rustc_serialize::json;
use std::thread;
use std::sync::{Arc, Mutex};

use datatype::{Command, Event};
use gateway::{Gateway, Interpret};
use http::{Server, ServerHandler};


/// The `Http` gateway parses `Command`s from the body of incoming requests.
pub struct Http {
    pub server: String,
}

impl Gateway for Http {
    fn initialize(&mut self, itx: Sender<Interpret>) -> Result<(), String> {
        let itx = Arc::new(Mutex::new(itx));
        let server = match HyperServer::http(&self.server.parse().expect("couldn't parse http address")) {
            Ok(server) => server,
            Err(err)   => return Err(format!("couldn't start http gateway: {}", err))
        };
        thread::spawn(move || {
            let (_, server) = server.handle(move |_| HttpHandler::new(itx.clone())).unwrap();
            server.run();
        });

        Ok(info!("HTTP gateway listening at http://{}", self.server))
    }
}


struct HttpHandler {
    itx:         Arc<Mutex<Sender<Interpret>>>,
    response_rx: Option<Receiver<Event>>
}

impl HttpHandler {
    fn new(itx: Arc<Mutex<Sender<Interpret>>>) -> ServerHandler<HttpStream> {
        ServerHandler::new(Box::new(HttpHandler { itx: itx, response_rx: None }))
    }
}

impl<T: Transport> Server<T> for HttpHandler {
    fn headers(&mut self, _: HyperRequest<T>) {}

    fn request(&mut self, body: Vec<u8>) {
        String::from_utf8(body).map(|body| {
            json::decode::<Command>(&body).map(|cmd| {
                info!("Incoming HTTP request command: {}", cmd);
                let (etx, erx)   = chan::async::<Event>();
                self.response_rx = Some(erx);
                self.itx.lock().unwrap().send(Interpret {
                    command:     cmd,
                    response_tx: Some(Arc::new(Mutex::new(etx))),
                });
            }).unwrap_or_else(|err| error!("http request parse json: {}", err))
        }).unwrap_or_else(|err| error!("http request parse string: {}", err))
    }

    fn response(&mut self) -> (StatusCode, Option<Vec<u8>>) {
        self.response_rx.as_ref().map_or((StatusCode::BadRequest, None), |rx| {
            rx.recv().map_or_else(|| {
                error!("on_response receiver error");
                (StatusCode::InternalServerError, None)
            }, |event| {
                json::encode(&event).map(|body| {
                    (StatusCode::Ok, Some(body.into_bytes()))
                }).unwrap_or_else(|err| {
                    error!("on_response encoding json: {:?}", err);
                    (StatusCode::InternalServerError, None)
                })
            })
        })
    }
}


#[cfg(test)]
mod tests {
    use chan;
    use crossbeam;
    use rustc_serialize::json;
    use std::path::Path;
    use std::thread;

    use super::*;
    use gateway::{Gateway, Interpret};
    use datatype::{Command, Event};
    use http::{AuthClient, Client, set_ca_certificates};


    #[test]
    fn http_connections() {
        set_ca_certificates(&Path::new("run/sota_certificates"));

        let (etx, erx) = chan::sync::<Event>(0);
        let (itx, irx) = chan::sync::<Interpret>(0);

        thread::spawn(move || Http { server: "127.0.0.1:8888".to_string() }.start(itx, erx));
        thread::spawn(move || {
            let _ = etx; // move into this scope
            loop {
                let interpret = irx.recv().expect("itx is closed");
                match interpret.command {
                    Command::StartDownload(ids) => {
                        let tx = interpret.response_tx.unwrap();
                        tx.lock().unwrap().send(Event::FoundSystemInfo(ids.first().unwrap().to_owned()));
                    }
                    _ => panic!("expected AcceptUpdates"),
                }
            }
        });

        crossbeam::scope(|scope| {
            for id in 0..10 {
                scope.spawn(move || {
                    let cmd     = Command::StartDownload(vec!(format!("{}", id)));
                    let client  = AuthClient::default();
                    let url     = "http://127.0.0.1:8888".parse().unwrap();
                    let body    = json::encode(&cmd).unwrap();
                    let resp_rx = client.post(url, Some(body.into_bytes()));
                    let resp    = resp_rx.recv().unwrap().unwrap();
                    let text    = String::from_utf8(resp).unwrap();
                    assert_eq!(json::decode::<Event>(&text).unwrap(),
                               Event::FoundSystemInfo(format!("{}", id)));
                });
            }
        });
    }
}
