use chan;
use chan::Sender;
use rustc_serialize::json;
use std::{env, thread};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use ws;
use ws::{listen, CloseCode, Handler, Handshake, Message, Sender as WsSender};
use ws::util::Token;

use datatype::{Command, Error, Event};
use super::gateway::{Gateway, Interpret};


pub struct Websocket {
    pub clients: Arc<Mutex<HashMap<Token, WsSender>>>
}

impl Gateway for Websocket {
    fn initialize(&mut self, itx: Sender<Interpret>) -> Result<(), String> {
        let addr = env::var("SOTA_WEBSOCKET_ADDR").unwrap_or("127.0.0.1:3012".to_string());
        info!("Opening websocket listener on {}", addr);

        let clients = self.clients.clone();
        thread::spawn(move || {
            listen(&addr as &str, |out| {
                WebsocketHandler {
                    out:     out,
                    itx:     itx.clone(),
                    clients: clients.clone()
                }
            }).expect("couldn't start websocket listener");
        });

        thread::sleep(Duration::from_secs(1)); // FIXME: ugly hack for blocking listen call
        Ok(info!("Websocket gateway started."))
    }

    fn pulse(&self, event: Event) {
        let json = encode(event);
        for (_, out) in self.clients.lock().unwrap().iter() {
            let _ = out.send(Message::Text(json.clone()));
        }
    }
}


pub struct WebsocketHandler {
    out:     WsSender,
    itx:     Sender<Interpret>,
    clients: Arc<Mutex<HashMap<Token, WsSender>>>
}

impl Handler for WebsocketHandler {
    fn on_message(&mut self, msg: Message) -> ws::Result<()> {
        debug!("received websocket message: {:?}", msg);
        msg.as_text().or_else(|err| {
            error!("websocket on_message text error: {}", err);
            Err(err)
        }).and_then(|msg| match decode(msg) {
            Ok(cmd) => Ok(self.forward_command(cmd)),

            Err(Error::Websocket(err)) => {
                error!("websocket on_message error: {}", err);
                Err(err)
            }

            Err(_)  => unreachable!()
        })
    }

    fn on_open(&mut self, _: Handshake) -> ws::Result<()> {
        let _ = self.clients.lock().unwrap().insert(self.out.token(), self.out.clone());
        Ok(debug!("new websocket client: {:?}", self.out.token()))
    }

    fn on_close(&mut self, code: CloseCode, _: &str) {
        let _ = self.clients.lock().unwrap().remove(&self.out.token());
        debug!("closing websocket client {:?}: {:?}", self.out.token(), code);
    }

    fn on_error(&mut self, err: ws::Error) {
        error!("websocket error: {:?}", err);
    }
}

impl WebsocketHandler {
    fn forward_command(&self, cmd: Command) {
        let (etx, erx) = chan::sync::<Event>(0);
        let etx        = Arc::new(Mutex::new(etx.clone()));
        self.itx.send(Interpret { command: cmd, response_tx: Some(etx) });

        let e = erx.recv().expect("websocket response_tx is closed");
        let _ = self.out.send(Message::Text(encode(e)));
    }
}

fn encode(event: Event) -> String {
    json::encode(&event).expect("Error encoding event into JSON")
}

fn decode(s: &str) -> Result<Command, Error> {
    Ok(try!(json::decode::<Command>(s)))
}


#[cfg(test)]
mod tests {
    use chan;
    use crossbeam;
    use rustc_serialize::json;
    use std::thread;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use ws;
    use ws::{connect, CloseCode};

    use gateway::{Gateway, Interpret};
    use datatype::{Command, Event};
    use super::*;


    #[test]
    fn websocket_connections() {
        let (etx, erx) = chan::sync::<Event>(0);
        let (itx, irx) = chan::sync::<Interpret>(0);
        let mut ws     = Websocket { clients: Arc::new(Mutex::new(HashMap::new())) };

        thread::spawn(move || { ws.start(itx, erx); });
        thread::spawn(move || {
            let _ = etx; // move into this scope
            loop {
                let interpret = irx.recv().expect("gtx is closed");
                match interpret.command {
                    Command::AcceptUpdates(ids) => {
                        let tx = interpret.response_tx.unwrap();
                        tx.lock().unwrap().send(Event::Error(ids.first().unwrap().to_owned()));
                    }
                    _ => panic!("expected AcceptUpdates"),
                }
            }
        });

        crossbeam::scope(|scope| {
            for id in 0..10 {
                scope.spawn(move || {
                    connect("ws://localhost:3012", |out| {
                        let text = format!(r#"{{ "variant": "AcceptUpdates", "fields": [["{}"]] }}"#, id);
                        out.send(text).unwrap();

                        move |msg: ws::Message| {
                            let ev: Event = json::decode(&format!("{}", msg)).unwrap();
                            assert_eq!(ev, Event::Error(format!("{}", id)));
                            out.close(CloseCode::Normal)
                        }
                    }).unwrap();
                });
            }
        });
    }
}
