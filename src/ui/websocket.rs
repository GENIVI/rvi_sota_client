use ws::{listen, Message, Sender as WsSender, Handler, Handshake, Result as WsResult, CloseCode};
use ws::util::Token;
use rustc_serialize::json;

use datatype::{Event, Command, Error};

use std::sync::mpsc::Sender;

use std::thread;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type SharedClients = Arc<Mutex<HashMap<Token, WsSender>>>;

pub struct WebsocketHandler {
    all_clients: SharedClients,
    out: WsSender,
    commands_tx: Sender<Command>
}

impl Handler for WebsocketHandler {
    fn on_open(&mut self, _: Handshake) -> WsResult<()> {
        self.all_clients.lock().unwrap().insert(self.out.token(), self.out.clone());
        Ok(())
    }

    fn on_message(&mut self, msg: Message) -> WsResult<()> {
        if let Message::Text(payload) = msg {
            match json::decode(&payload) {
                Ok(command) => { let _ = self.commands_tx.send(command); },
                Err(e) => {
                    let err = format!("Invalid command: {}. Reason: {}", payload, e);
                    error!("{}", err);
                    let _ = self.out.send(json::encode(&Event::Error(err)).unwrap());
                }
            }
        };
        Ok(())
    }

    fn on_close(&mut self, _: CloseCode, _: &str) {
        self.all_clients.lock().unwrap().remove(&self.out.token());
    }
}

pub fn spawn_websocket_server_async(addr: &'static str, ctx: Sender<Command>, all_clients: SharedClients) -> Result<thread::JoinHandle<()>, Error> {
    Ok(try!(thread::Builder::new().name("ui".to_string()).spawn(move || {
        spawn_websocket_server(addr, ctx, all_clients).unwrap_or_else(|e| error!("{}", e))
    })))
}

pub fn spawn_websocket_server(addr: &'static str, ctx: Sender<Command>, all_clients: SharedClients) -> Result<(), Error> {
    listen(addr, move |out| {
        WebsocketHandler {
            all_clients: all_clients.clone(),
            out: out,
            commands_tx: ctx.clone()
        }
    }).map_err(Error::Websocket)
}
