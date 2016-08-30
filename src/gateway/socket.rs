use chan;
use chan::Sender;
use rustc_serialize::json;
use std::io::{BufReader, Read, Write};
use std::net::Shutdown;
use std::sync::{Arc, Mutex};
use std::{fs, thread};

use datatype::{Command, Error, Event};
use super::{Gateway, Interpret};
use unix_socket::{UnixListener, UnixStream};


/// The `Socket` gateway is used for communication via Unix Domain Sockets.
pub struct Socket {
    pub path: String
}

impl Gateway for Socket {
    fn initialize(&mut self, itx: Sender<Interpret>) -> Result<(), String> {
        let itx = Arc::new(Mutex::new(itx));
        let _   = fs::remove_file(&self.path);

        let server = match UnixListener::bind(self.path.clone()) {
            Ok(server) => server,
            Err(err)   => return Err(format!("couldn't start socket gateway: {}", err))
        };

        thread::spawn(move || {
            for input in server.incoming() {
                if let Err(err) = input {
                    error!("couldn't read socket input: {}", err);
                    continue
                }
                let mut stream = input.unwrap();
                let itx = itx.clone();

                thread::spawn(move || {
                    let resp = handle_client(&mut stream, itx)
                        .map(|ev| json::encode(&ev).expect("couldn't encode Event").into_bytes())
                        .unwrap_or_else(|err| format!("{}", err).into_bytes());

                    stream.write_all(&resp)
                        .unwrap_or_else(|err| error!("couldn't write to socket: {}", err));
                    stream.shutdown(Shutdown::Write)
                        .unwrap_or_else(|err| error!("couldn't close socket for writing: {}", err))
                });
            }
        });

        Ok(info!("Unix Domain Socket gateway listening at {}.", self.path))
    }
}

fn handle_client(stream: &mut UnixStream, itx: Arc<Mutex<Sender<Interpret>>>) -> Result<Event, Error> {
    info!("New domain socket connection");
    let mut reader = BufReader::new(stream);
    let mut input  = String::new();
    try!(reader.read_to_string(&mut input));
    debug!("socket input: {}", input);

    let cmd = try!(input.parse::<Command>());
    let (etx, erx) = chan::async::<Event>();
    itx.lock().unwrap().send(Interpret {
        command:     cmd,
        response_tx: Some(Arc::new(Mutex::new(etx))),
    });
    erx.recv().ok_or(Error::Socket("internal receiver error".to_string()))
}


#[cfg(test)]
mod tests {
    use chan;
    use crossbeam;
    use rustc_serialize::json;
    use std::thread;
    use std::io::{Read, Write};
    use std::net::Shutdown;
    use std::time::Duration;

    use datatype::{Command, Event};
    use gateway::{Gateway, Interpret};
    use super::*;
    use unix_socket::UnixStream;


    #[test]
    fn unix_domain_socket_connections() {
        let (etx, erx) = chan::sync::<Event>(0);
        let (itx, irx) = chan::sync::<Interpret>(0);

        thread::spawn(move || Socket { path: "/tmp/sota.socket".to_string() }.start(itx, erx));
        thread::sleep(Duration::from_millis(100)); // wait until socket is created

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
                    let mut stream = UnixStream::connect("/tmp/sota.socket").expect("couldn't connect to socket");
                    let _ = stream.write_all(&format!("acc {}", id).into_bytes()).expect("couldn't write to stream");
                    stream.shutdown(Shutdown::Write).expect("couldn't shut down writing");

                    let mut resp = String::new();
                    stream.read_to_string(&mut resp).expect("couldn't read from stream");
                    let ev: Event = json::decode(&resp).expect("couldn't decode json event");
                    assert_eq!(ev, Event::Error(format!("{}", id)));
                });
            }
        });
    }
}
