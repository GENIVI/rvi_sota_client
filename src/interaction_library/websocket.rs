use chan;
use chan::Sender;
use rustc_serialize::{json, Decodable, Encodable};
use std::{env, thread};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use ws;
use ws::{listen, Sender as WsSender, Handler, Message, Handshake, CloseCode};
use ws::util::Token;

use super::gateway::{Gateway, Interpret};
use datatype::Error;


type Clients = Arc<Mutex<HashMap<Token, WsSender>>>;


pub struct Websocket;

impl<C, E> Gateway<C, E> for Websocket
    where C: Decodable + Send + Clone + Debug + 'static,
          E: Encodable + Send + Clone + Debug + 'static,
{
    fn new(itx: Sender<Interpret<C, E>>) -> Result<Self, String> {
        let (etx, erx) = chan::sync::<E>(0);
        let etx        = Arc::new(Mutex::new(etx.clone()));
        let clients    = Arc::new(Mutex::new(HashMap::new()));
        let addr       = env::var("OTA_PLUS_CLIENT_WEBSOCKET_ADDR").unwrap_or("127.0.0.1:3012".to_string());

        let rx_clients = clients.clone();
        thread::spawn(move || {
            loop {
                match erx.recv() {
                    Some(e) => send_response(rx_clients.clone(), e),
                    None    => panic!("all websocket response transmitters are closed")
                }
            }
        });

        let (start_tx, start_rx) = chan::sync::<Result<(), ws::Error>>(0);
        thread::spawn(move || {
            info!("Opening websocket listener on {}", addr);
            start_tx.send(listen(&addr as &str, |sender| {
                WebsocketHandler {
                    clients: clients.clone(),
                    sender:  sender,
                    itx:     itx.clone(),
                    etx:     etx.clone(),
                }
            }));
        });

        let tick = chan::tick_ms(1000); // FIXME: ugly hack for blocking call
        chan_select! {
            tick.recv() => return Ok(Websocket),
            start_rx.recv() -> outcome => match outcome {
                Some(outcome) => match outcome {
                    Ok(_)    => return Ok(Websocket),
                    Err(err) => return Err(format!("couldn't open websocket listener: {}", err))
                },
                None => panic!("expected websocket start outcome")
            }
        }
    }
}

pub struct WebsocketHandler<C, E>
    where C: Decodable + Send + Clone + Debug + 'static,
          E: Encodable + Send + Clone + Debug + 'static,
{
    clients: Clients,
    sender:  WsSender,
    itx:     Sender<Interpret<C, E>>,
    etx:     Arc<Mutex<Sender<E>>>,
}

impl<C, E> Handler for WebsocketHandler<C, E>
    where C: Decodable + Send + Clone + Debug + 'static,
          E: Encodable + Send + Clone + Debug + 'static,
{
    fn on_message(&mut self, msg: Message) -> ws::Result<()> {
        match decode(&format!("{}", msg)) {
            Ok(cmd) => Ok(self.itx.send(Interpret { command: cmd, response_tx: Some(self.etx.clone()) })),

            Err(Error::WebsocketError(err)) => {
                error!("websocket decode error: {}", err);
                Err(err)
            }

            Err(_) => unreachable!()
        }
    }

    fn on_open(&mut self, _: Handshake) -> ws::Result<()> {
        let mut map = self.clients.lock().expect("Poisoned map lock -- can't continue");
        let _       = map.insert(self.sender.token(), self.sender.clone());
        Ok(())
    }

    fn on_close(&mut self, _: CloseCode, _: &str) {
        let mut map = self.clients.lock().expect("Poisoned map lock -- can't continue");
        let _       = map.remove(&self.sender.token().clone());
    }
}


fn encode<E: Encodable>(e: E) -> String {
    json::encode(&e).expect("Error encoding event into JSON")
}

fn decode<C: Decodable>(s: &str) -> Result<C, Error> {
    Ok(try!(json::decode::<C>(s)))
}

fn send_response<E: Encodable>(clients: Clients, e: E) {
    let txt = encode(e);
    let map = clients.lock().expect("Poisoned map lock -- can't continue");
    for (_, sender) in map.iter() {
        let _ = sender.send(Message::Text(txt.clone()));
    }
}
