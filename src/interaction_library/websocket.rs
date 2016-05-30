use rustc_serialize::{json, Decodable, Encodable};
use std::env;
use std::collections::HashMap;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use ws::util::Token;
use ws::{listen, Sender as WsSender, Handler, Message, Handshake, CloseCode};
use ws;

use super::gateway::{Gateway, Interpret};
use datatype::Error;


type Clients = Arc<Mutex<HashMap<Token, WsSender>>>;

pub struct WebsocketHandler {
    out:     WsSender,
    sender:  Sender<String>,
    clients: Clients,
}

impl Handler for WebsocketHandler {
    fn on_message(&mut self, msg: Message) -> ws::Result<()> {
        Ok(match self.sender.send(format!("{}", msg)) {
            Ok(_) => {}
            Err(e) => error!("Error forwarding message from WS: {}", e),
        })
    }

    fn on_open(&mut self, _: Handshake) -> ws::Result<()> {
        let mut map = self.clients.lock().expect("Poisoned map lock -- can't continue");
        let _ = map.insert(self.out.token(), self.out.clone());
        Ok(())

    }

    fn on_close(&mut self, _: CloseCode, _: &str) {
        let mut map = self.clients.lock().expect("Poisoned map lock -- can't continue");
        let _ = map.remove(&self.out.token().clone());
    }
}


#[derive(Clone)]
pub struct Websocket {
    clients:  Clients,
    receiver: Arc<Mutex<Receiver<String>>>,
}

impl Websocket {
    fn get_line(&self) -> String {
        let rx = self.receiver.lock().expect("Poisoned rx lock -- can't continue");
        match rx.recv() {
            Ok(line) => line,
            Err(err) => {
                error!("Couldn't fetch from WS receiver: {:?}", err);
                "".to_string()
            }
        }
    }

    fn put_line(&self, s: String) {
        let map = self.clients.lock().expect("Poisoned map lock -- can't continue");
        for (_, out) in map.iter() {
            let _ = out.send(Message::Text(s.clone()));
        }
    }
}

impl<C, E> Gateway<C, E> for Websocket
    where C: Decodable + Send + Clone + 'static,
          E: Encodable + Send + Clone + 'static,
{
    fn new() -> Websocket {
        let (tx, rx) = mpsc::channel();
        let clients  = Arc::new(Mutex::new(HashMap::new()));
        let moved    = clients.clone();
        let addr     = env::var("OTA_PLUS_CLIENT_WEBSOCKET_ADDR")
                          .unwrap_or("127.0.0.1:3012".to_string());

        thread::spawn(move || {
            listen(&addr as &str, |out| {
                WebsocketHandler {
                    out:     out,
                    sender:  tx.clone(),
                    clients: moved.clone(),
                }
            })
        });

        Websocket {
            clients:  clients,
            receiver: Arc::new(Mutex::new(rx)),
        }
    }

    fn next(&self) -> Option<Interpret<C, E>> {
        match decode(&self.get_line()) {
            Ok(cmd) => {
                let (etx, erx): (Sender<E>, Receiver<E>) = mpsc::channel();
                let clone = self.clone();
                thread::spawn(move || {
                    match erx.recv() {
                        Ok(e) => clone.put_line(encode(e)),
                        Err(err) => error!("Error receiving event: {:?}", err),
                    }
                });
                Some(Interpret {
                    cmd: cmd,
                    etx: Some(Arc::new(Mutex::new(etx))),
                })
            }

            Err(err) => {
                error!("Error decoding JSON: {}", err);
                None
            }
        }
    }
}

fn encode<E: Encodable>(e: E) -> String {
    json::encode(&e).expect("Error encoding event into JSON")
}

fn decode<C: Decodable>(s: &str) -> Result<C, Error> {
    Ok(try!(json::decode::<C>(s)))
}
