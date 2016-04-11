use std::collections::HashMap;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use ws::util::Token;
use ws::{listen, Sender as WsSender, Handler, Message, Handshake, CloseCode};
use ws;

use super::gateway::Gateway;
use super::parse::Parse;
use super::print::Print;


type Clients = Arc<Mutex<HashMap<Token, WsSender>>>;

pub struct WebsocketHandler {
    out:     WsSender,
    sender:  Sender<String>,
    clients: Clients
}

impl Handler for WebsocketHandler {

    fn on_message(&mut self, msg: Message) -> ws::Result<()> {
        Ok(self.sender.send(format!("{}", msg)).unwrap())
    }

    fn on_open(&mut self, _: Handshake) -> ws::Result<()> {
        let mut map = (*self.clients).lock().unwrap();
        let _ = map.insert(self.out.token(), self.out.clone());
        Ok(())

    }

    fn on_close(&mut self, _: CloseCode, _: &str) {
        let mut map = (*self.clients).lock().unwrap();
        let _ = map.remove(&self.out.token().clone());
    }
}

pub struct Websocket {
    clients:  Clients,
    receiver: Receiver<String>,
}

impl<C, E> Gateway<C, E> for Websocket
    where
    C: Parse + Send + 'static, E: Print + Send + 'static {

    fn new() -> Websocket {

        fn spawn(tx: Sender<String>, clients: Clients) {

            thread::spawn(move || {
                listen("127.0.0.1:3012", |out| {
                    WebsocketHandler {
                        out:     out,
                        sender:  tx.clone(),
                        clients: clients.clone(),
                    }
                })
            });

        }

        let (tx, rx) = mpsc::channel();
        let clients  = Arc::new(Mutex::new(HashMap::new()));

        spawn(tx, clients.clone());

        Websocket {
            clients:  clients.clone(),
            receiver: rx,
        }
    }

    fn get_line(&self) -> String {
        self.receiver.recv().unwrap()
    }

    fn put_line(&self, s: String) {
        let map = (*self.clients).lock().unwrap();
        let _ = map
            .values()
            .map(|out| out.send(Message::Text(s.clone())));
    }

}
